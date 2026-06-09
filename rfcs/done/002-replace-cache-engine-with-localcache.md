# RFC 002 — Replace the in-house cache engine with localcache

**Status.** Implemented (v0.23.0)
**Tracks.** Retirement of the in-repo `file-feature-cache`
engine in favour of the published `localcache` crate, v0.20.x.
The `arama-cache` facade crate is kept; only its engine layer
is swapped.
**Touches.** `crates/engine/file-feature-cache/` (removed at
the end of the migration), `crates/cache/` (internals rewritten
on top of `localcache`; public API preserved), workspace
`Cargo.toml` (drop `file-feature-cache`, `r2d2`,
`r2d2_sqlite`, `sha2`; add `localcache = "0.19"`,
`blake3 = "1"`, upgrade `rusqlite` to `0.39` to match
localcache's bundled version),
`env/src/...` (cache DB path naming), on-disk cache database
format (breaking; see § Data migration).

## Summary

arama caches AI inference results (CLIP / wav2vec2 feature
vectors) and thumbnails per media file in SQLite, via the
in-repo `file-feature-cache` engine (r2d2 connection pools,
SHA-256 fingerprinting, a `CacheExtension` trait for
per-media-type tables). `localcache` 0.20 is a published,
maintained crate covering the same problem — payloads tied to
local files with freshness detection — with capabilities the
in-house engine lacks: pluggable change detection (metadata-
fast-path), namespaces, payload versioning, TTL/LRU, a query
builder, export/import portability, and an optional async
engine.

This RFC swaps the engine underneath `arama-cache` while
keeping `ImageCacheWriter` / `VideoCacheWriter` /
`*CacheReader` and the `LookupResult` contract stable, so the
five consumer sites (`app`, `arama-ai` ×2, `ui/widgets`
dialogs ×2) need no change in this RFC. `file-feature-cache`
(~1.5 k lines plus tests) is deleted, and its "todo: publish as
crate" debt is resolved by adopting the crate that already
exists.

## Motivation

1. **Stop maintaining a storage engine.** Fingerprinting,
   pooling, schema migration, and WAL tuning are not arama's
   domain. Every `file-feature-cache` bug is arama's bug.
2. **Faster freshness checks.** The in-house engine always
   computes a full SHA-256 per check. `localcache`'s
   `MetadataThenFullHash` skips hashing when size+mtime match —
   the common case when re-opening a directory of thousands of
   media files.
3. **Payload versioning.** When a model or embedding pipeline
   changes, `payload_version` + `purge_stale_versions()` gives
   a principled invalidation story. Today arama has none
   (stale embeddings survive model upgrades silently).
4. **Operational tooling.** `localcache-cli` (stats / list /
   inspect / export) replaces "open the SQLite file by hand"
   for debugging user cache issues.
5. **Documentation-language compliance.** The engine and facade
   are currently documented in Japanese; the project standard
   is English. The rewritten facade internals are documented in
   English as part of this work.
6. **Shared maintenance.** Same author (nabbisen); arama's
   needs (see § Questions for the localcache author) can land
   upstream and benefit other users.

## Current state (as-is analysis)

### Engine: `crates/engine/file-feature-cache`

- `files` table: canonicalized absolute path (UNIQUE),
  `file_hash` (SHA-256), `mtime_ns`, timestamps.
- `CacheExtension` trait lets `arama-cache` inject extra tables
  at migration time.
- `CacheWriter<E>` (single write conn) / `CacheReader<E>`
  (r2d2 read pool, `read_conns` configurable); `refresh_all` /
  `check_all` parallelize fingerprinting with rayon.
- `DbLocation::{AppCache, WorkDir, Custom}` resolves the DB
  path.

### Facade: `crates/cache` (`arama-cache`)

- `MediaExtension` adds `thumbnails(id, thumbnail_path)`,
  `image_features(id, clip_vector BLOB)`,
  `video_features(id, clip_vector BLOB, wav2vec2_vector BLOB)`,
  all `ON DELETE CASCADE` from `files`.
- Vectors stored as raw little-endian `f32` blobs
  (`codec::vec_to_blob` / `blob_to_vec`).
- Thumbnails generated here (224×224 JPEG via `image`; video
  posters via ffmpeg) into `thumbnail_dir`, named by row `id`.
- Directory-scoped queries (`all`, `all_in_dir`,
  `all_in_dir_and_sub_dirs`) implemented with SQL `GLOB`
  patterns, driven by the user setting `CacheLookupStrategy`
  (`Everywhere` / `CurrentDirAndSubDirs` / `CurrentDirOnly`).
- `LookupResult::{Hit, Invalidated, Miss}` is the contract the
  app logic relies on.

### Consumers (unchanged by this RFC)

- `app/src/core/update.rs` — bulk thumbnail upsert on startup
  (`upsert_all`, vectors `None`).
- `crates/ai/.../embeddings.rs` — session writer; lookup-then-
  upsert per file during embedding runs.
- `crates/ai/.../video_similarity_pipeline.rs` — video
  reader/writer.
- `crates/ui/widgets/.../similar_media.rs`,
  `similar_pairs_dialog.rs` — read-side, strategy-scoped
  queries feeding HNSW similarity search.

## Target design (to-be)

### External design

User-visible behaviour is unchanged, with two exceptions:

1. **One-time cache rebuild** (or migration; § Data migration):
   the on-disk format changes.
2. **Faster directory re-open** thanks to metadata-fast-path
   freshness checks (no full hash of unchanged files).

The `CacheLookupStrategy` setting keeps its exact semantics.

### Internal design

#### Payload model

`localcache` stores one typed payload per file. The extension
tables collapse into serde structs (bincode codec, the
default — compact and stable per localcache's migration notes):

```rust
// crates/cache/src/types.rs (additions; existing pub types kept)
#[derive(Serialize, Deserialize, ...)]
pub(crate) struct ImagePayload {
    pub thumbnail_path: Option<String>,
    pub clip_vector: Option<Vec<f32>>,
}

#[derive(Serialize, Deserialize, ...)]
pub(crate) struct VideoPayload {
    pub thumbnail_path: Option<String>,
    pub clip_vector: Option<Vec<f32>>,
    pub wav2vec2_vector: Option<Vec<f32>>,
}
```

Raw-blob codec (`core/codec.rs`) is deleted; bincode's
`Vec<f32>` encoding replaces it. (Size note: bincode legacy
config encodes `f32` as 4 LE bytes + a length prefix —
effectively identical footprint to the current blobs.)

#### Engine configuration

One SQLite database file, two namespaces:

```rust
fn image_engine(db: &Path) -> Result<CacheEngine<ImagePayload>> {
    CacheEngine::<ImagePayload>::builder()
        .database(db)
        .namespace("image")
        .change_detection(ChangeDetectionMode::MetadataThenFullHash)
        .payload_version(IMAGE_PAYLOAD_VERSION) // starts at 1
        .build()
}
// video_engine analogous, namespace "video",
// VIDEO_PAYLOAD_VERSION starts at 1
```

- `JournalMode` stays at localcache's WAL default (parity with
  the current engine).
- No TTL, no `max_entries` initially: arama's cache is a
  derived index over the user's media library; eviction by
  recency would silently re-trigger expensive embedding work.
  Disk-pressure handling is a follow-up RFC (the `disk-space`
  dependency todo).
- `payload_version` constants live in `arama-cache` and must be
  bumped whenever the embedding pipeline or thumbnail format
  changes; `purge_stale_versions()` runs at engine open.

#### Facade mapping

Public API of `arama-cache` is preserved. Internal mapping:

| Current (file-feature-cache) | New (localcache) |
|---|---|
| `CacheWriter::as_session(CacheConfig)` | `CacheEngine::builder()...build()` wrapped in `ConnectionPool` |
| `CacheWriter::onetime(DbLocation)` | same engine construction; "onetime" remains a facade-level convenience |
| `refresh(path)` + extension-table upserts | `get_if_fresh` → merge new fields into existing payload → `set(path, payload)` |
| `refresh_all` / `upsert_all` (rayon + serial writes) | thumbnail/fingerprint work parallel in facade (rayon, as today); writes via `batch_set` |
| `check(path)` | `check_status(path)` |
| `lookup → Hit/Invalidated/Miss` | `check_status` → `CacheStatus` mapped to `LookupResult`; `Hit` carries the deserialized payload from `get_if_fresh` |
| `delete(path)` | `remove(path)` |
| `list_paths()` | `keys()` |
| `all_in_dir*` GLOB queries | `query().path_like(...)` + facade-side filtering (see Q1) |
| `DbLocation` enum | kept in facade (re-exported type today), resolving to a `PathBuf` handed to `.database(...)` |
| `CacheExtension` trait | deleted — payload structs subsume it |

`LookupResult` nuance preserved: today `Invalidated` also
*deletes* the stale row. With localcache the facade calls
`remove()` on `CacheStatus::Stale` before returning
`Invalidated`, keeping observable semantics identical.

Thumbnail file naming: row `id` no longer exists, so thumbnails
are named by a hash of the canonicalized path (blake3, already
in the dependency tree via localcache). Existing thumbnail
files become orphans and are cleaned by the migration step.

#### Concurrency model — the one real regression risk

The current engine offers a true multi-connection read pool;
read-side `check_all`/`lookup` run concurrently. localcache's
`ConnectionPool` is `Arc<Mutex<CacheEngine>>` — all operations
serialize on one connection.

Assessment:

- Write paths are already serialized today (single write conn);
  no regression.
- Read-heavy paths (gallery load, similarity dialogs) currently
  fan out lookups across `read_conns`. Under localcache these
  serialize, but `batch_get_fresh` amortizes per-call locking
  and SQLite point lookups on an indexed path column are
  microseconds; the dominant costs in these paths are thumbnail
  I/O and HNSW search, not SQLite.
- **Decision gate:** Phase B includes a benchmark (10 k cached
  entries, `CurrentDirAndSubDirs` query + full lookup fan-out)
  comparing old vs new. Acceptance: ≤ 1.5× wall time on the
  cache portion. If it fails, escalate Q2 upstream before
  proceeding.

#### Dependency changes

| Workspace dep | Change |
|---|---|
| `file-feature-cache` | removed (crate deleted) |
| `r2d2`, `r2d2_sqlite` | removed |
| `sha2` | removed (blake3 inside localcache supersedes) |
| `rusqlite` | removed from `arama-cache`; remains only if the migration shim (§ below) needs to read the old DB — the shim may pin its own copy |
| `localcache` | added, `"0.20"`, default features (no `async`/`json` initially); `blake3 = "1"` also added (thumbnail naming) |

Note: localcache pins `rusqlite 0.39` (bundled); arama's
remaining direct uses are at `0.38`. After this RFC,
`arama-cache` has no direct rusqlite dependency, so the
duplicate-bundled-sqlite problem disappears with the old
engine's deletion. The migration shim (temporary) tolerates one
release of duplication.

### Program design — file-level plan

```
crates/cache/src/
  lib.rs              — English rustdoc rewrite; same re-exports
  types.rs            — + ImagePayload/VideoPayload (private)
  core.rs             — engine.rs replaces codec.rs
  core/engine.rs      — builder helpers, DbLocation resolution,
                        payload_version constants, status→LookupResult map
  core/image.rs       — ImageCacheWriter/Reader on CacheEngine<ImagePayload>
  core/video.rs       — VideoCacheWriter/Reader on CacheEngine<VideoPayload>
  core/query.rs       — strategy-scoped listing via QueryBuilder
  core/thumbnail.rs   — unchanged except id→path-hash naming
  core/migrate.rs     — one-time v1→v2 importer (feature-gated,
                        removed after one release cycle)
  tests.rs / tests/   — see § Testing
crates/engine/file-feature-cache/  — deleted in the final phase
```

Each file stays under the 300-ELOC guideline; `core/image.rs`
and `core/video.rs` shrink versus today (no SQL).

## Data migration

The old schema (relational, blobs, row-id thumbnails) and the
new one (localcache entries, bincode payloads) are not
compatible. Two options were considered:

- **(a) Recompute.** Delete old DB; thumbnails and embeddings
  regenerate lazily. Simple, but embeddings cost real GPU/CPU
  time on large libraries — a poor user experience for an
  upgrade.
- **(b) One-time import (chosen).** On first run, if the old DB
  file exists and the new one does not, `core/migrate.rs`
  reads `files ⋈ thumbnails ⋈ image_features / video_features`
  from the old DB and `batch_set`s payloads into the new
  engine. File freshness is re-validated by localcache on
  insert/first read, so stale rows drop out naturally.
  Thumbnails are re-linked (path stored in payload) and
  renamed to the new hash-based names; orphans deleted. The
  old DB file is renamed `*.v1.bak` (kept one release, then
  removal noted in CHANGELOG).

New DB filename gets a `-v2` suffix (exact naming handled in
`env/src/...` path helpers) so old and new never collide.

## Phased plan

1. **Phase A — facade rewrite behind the existing tests.**
   Implement `core/engine.rs`, image, video on localcache;
   port `crates/cache/tests/integration_tests.rs` to run
   against the new internals unchanged (they exercise the
   public API only — this is the compatibility proof).
2. **Phase B — queries + benchmark.** Strategy-scoped queries;
   concurrency benchmark; decision gate (Q2 escalation if
   needed).
3. **Phase C — migration shim** + `env` path changes; manual
   upgrade test with a populated v1 cache.
4. **Phase D — deletion.** Remove `crates/engine/file-feature-cache`,
   prune workspace deps, English-docs sweep of `arama-cache`.

Phases A–B and C–D can land as two releases if review prefers.

## Testing

Per project guidelines, tests validate the design spec:

- **Contract tests (exist today, kept verbatim):**
  `crates/cache/tests/integration_tests.rs` — upsert/lookup/
  Hit/Invalidated/Miss/delete round-trips for image and video.
  Passing unchanged is the API-stability criterion.
- **New spec tests** (in `crates/cache/src/tests.rs`, split per
  the line-count rule if needed):
  - payload round-trip: vectors survive bincode encode/decode
    bit-exactly;
  - namespace isolation: image and video entries for the same
    path do not collide;
  - `CacheLookupStrategy` scoping: `CurrentDirOnly` excludes
    subdirectory entries; `CurrentDirAndSubDirs` includes them
    (fixture tree under `tempfile`);
  - stale handling: touched file ⇒ `Invalidated` and entry
    removed;
  - payload-version purge: bumped constant ⇒ old entries gone
    after open.
- **Migration tests:** fixture v1 SQLite file → import →
  entries present, thumbnails renamed, `.v1.bak` left behind.
- Benchmark (Phase B) is a `criterion` bench, not CI-gating.

## Risks and mitigations

| Risk | Mitigation |
|---|---|
| Read-path serialization slows large galleries | Phase B benchmark gate; batch APIs; Q2 upstream |
| `path_like` (SQL LIKE) can't express "dir, non-recursive" exactly (current code uses GLOB + NOT GLOB) | facade post-filters by `Path::parent()` equality; correctness identical, minor over-fetch; Q1 asks for native glob/depth predicates |
| bincode payload evolution (adding fields later breaks decode) | treat payload struct changes as `payload_version` bumps — documented in `core/engine.rs` |
| Migration importer bugs corrupt user caches | importer is read-only on the v1 file; new DB written separately; failure ⇒ fall back to recompute path |
| localcache pre-1.0 API churn | pin `=0.20.x`; author reachable; migration guide exists upstream |

## Alternatives considered

- **Publish `file-feature-cache` as planned (the in-code todo).**
  Rejected: it would compete with localcache, same author,
  smaller feature set — pure duplication.
- **Keep the engine, adopt only blake3/metadata fast-path.**
  Rejected: replicates localcache feature-by-feature over time.
- **Two DB files instead of namespaces.** Rejected: namespaces
  keep the single-file "easy to ship or delete" property and
  the CLI can inspect both media types at once.
- **`json` codec for payloads** (enables `field_eq` queries on
  vectors' metadata). Rejected for now: vectors as JSON arrays
  are ~3× larger; arama has no payload-field query need.

## Questions for the localcache author

- **Q1.** Would a `path_glob(...)` and/or
  `path_in_dir(dir, recursive: bool)` predicate on
  `QueryBuilder` be acceptable upstream? arama needs exact
  non-recursive directory scoping; LIKE + post-filter works but
  is wasteful on `Everywhere`-sized caches.
- **Q2.** Any roadmap for concurrent readers (e.g. a real
  read-only connection pool, leveraging WAL)? Our gallery path
  fans out thousands of point lookups.
- **Q3.** Is bincode payload layout guaranteed stable across
  localcache minor versions (the 0.14 notes say
  `config::legacy()` — is that a permanent commitment)? This
  determines whether we must bump `payload_version` on
  localcache upgrades.
- **Q4.** `cleanup_missing_files()` semantics: does it
  canonicalize before comparing? arama stores canonicalized
  paths and runs on case-insensitive filesystems (Windows).

## Implementation notes (v0.23.0)

These notes record the as-built state for readers comparing the
design above to the shipped code.

**Target version raised to v0.20.** The RFC was written against
v0.16. Before implementation began, the localcache author
resolved all four open questions across v0.17–v0.20:

- `path_in_dir(dir, recursive: bool)` and `path_glob` landed
  in v0.18, replacing the facade's LIKE + post-filter workaround
  noted in the Risks table.
- `ReadPool<T>` — a cloneable, `Send+Sync` pool of read-only
  WAL connections — landed in v0.19. The Phase B benchmark gate
  was therefore not needed: all reader handles (standalone and
  via `as_reader()`) hold a `ReadPool` sized to
  `CacheConfig::read_conns`, giving full read concurrency. The
  serialization risk noted in the Risks table is obsolete.
- Bincode wire-format stability was formally guaranteed and
  golden-fixture tested in v0.19.
- Path-handling semantics (canonicalization, deleted-file
  fallback, `cleanup_missing_files`, Windows case-insensitivity)
  were documented and regression-tested in v0.19.
- `mtime` nanosecond precision landed in v0.20: `localcache`
  now stores `mtime` as nanoseconds since the UNIX epoch, so
  `MetadataThenFullHash` has no same-second blind window.
  The integration tests use different-length overwrite content
  (size change) for unambiguous detection independent of timing,
  which remains a sound practice regardless of mtime precision.

**Phases A–D implemented in one pass.** The all-phases-one-
release option was taken (single v0.23.0 change). The migration
shim (`core/migrate.rs`) is present and wired to app startup;
it is scheduled for removal in the next release cycle, at which
point `rusqlite` drops out of `arama-cache`'s deps.

**`Invalidated` no longer deletes the stale row.** The v1
engine deleted the row when `Invalidated` was returned; readers
in v2 are read-only, so the stale row is overwritten on the
next writer `upsert`. Observable behaviour at all five consumer
sites is identical (`Invalidated` and `Miss` are both treated as
"recompute").

**Thumbnail regeneration on file change.** v1 checked only
`if !dest.exists()` and skipped regeneration when the thumbnail
was present, even if the source file had changed. v2 also
regenerates the thumbnail whenever `status != Fresh`. This is
a correctness fix, not a behaviour change visible at the API
boundary.

**`TempFile::with_suffix` added to the test helper.** The
thumbnail integration test requires a source file with a `.jpg`
extension so that `image::open` can infer the format. The
pre-existing `TempFile::new` creates extensionless files, which
failed in the same way on the original code (pre-existing test
environment issue). A `with_suffix` constructor was added; the
thumbnail test is updated to use it.

## Open questions

- Disk-pressure / cache-size management (revives the
  `disk-space` todo) — follow-up RFC once this lands.
- Whether `AsyncCacheEngine` (feature `async`) should replace
  the `Task::perform(async move { ... })` +
  blocking-engine pattern in `app/src/core/update.rs` —
  follow-up RFC; requires no facade change thanks to this
  RFC's encapsulation.

## Out of scope

- Any change to the embedding pipelines, HNSW search, or
  thumbnail dimensions.
- The five consumer call sites (compile untouched).
- i18n / UI concerns (see RFC 001).
