# RFC 016 — Cache capacity and disk-pressure management

**Status.** Proposed
**Tracks.** RFC 002 follow-up and RFC 015 split-path follow-up: define how
arama measures cache storage, exposes cache-size controls, and prunes cache
entries under an explicit user-visible policy.
**Touches.** `crates/cache/`, `crates/cache/src/types.rs`,
`crates/ui/main/src/core/views/cache_page*`, `app/src/core/update/cache.rs`,
`crates/i18n/src/en.rs`, `crates/i18n/src/ja.rs`,
`docs/src/users/cache.md`, `docs/src/dev/workspace.md`,
`CHANGELOG.md`, `rfcs/README.md`.

## Summary

arama can show and clear per-directory cache entries, but it does not yet have
a cache-capacity policy. RFC 002 named disk-pressure/cache-size management as a
follow-up, and RFC 015 deliberately kept that work separate from v1 migration
retirement.

This RFC proposes a first cache-capacity pass with three pieces:

1. Measure the actual cache footprint separately from indexed source media size.
2. Add explicit user-visible pruning controls.
3. Keep automatic background eviction and persisted cache-limit policy out of
   the first implementation.

## Why

The current Cache page `Size` column aggregates the recorded sizes of cached
source media files. That is useful for understanding which directories are
indexed, but it is not the number of bytes consumed by arama's cache. Actual
cache footprint is closer to:

- `.arama-cache/cache-v2.sqlite`
- SQLite sidecar files such as `cache-v2.sqlite-wal` and `cache-v2.sqlite-shm`
- `.arama-cache/thumbnail/` contents

Without a distinct footprint model, cache-size and disk-pressure decisions
would be based on the wrong number.

## Design

### Part A — Cache footprint API

Add an `arama-cache` maintenance API that can report cache storage footprint:

```rust
pub struct CacheFootprint {
    pub database_bytes: u64,
    pub database_sidecar_bytes: u64,
    pub thumbnail_bytes: u64,
    pub total_bytes: u64,
}
```

Footprint measurement should:

- use filesystem metadata for the database file;
- include SQLite sidecars for the active database path (`-wal`, `-shm`) when
  present;
- recursively sum the configured thumbnail directory when present;
- return zero for missing files/directories rather than treating absence as an
  error.

The existing `DirCacheSummary::total_size` remains source-media size. The Cache
page should label it clearly as media size if both concepts are displayed.

### Part B — Prune API

Add an explicit prune API that removes reclaimable cache bytes toward a one-off
target:

```rust
pub struct CachePruneRequest {
    pub max_bytes: u64,
}

pub struct CachePruneReport {
    pub before: CacheFootprint,
    pub after: CacheFootprint,
    pub target_reached: bool,
    pub unreclaimable_bytes: u64,
    pub removed_entries: usize,
    pub removed_recorded_thumbnail_bytes: u64,
    pub removed_orphan_thumbnail_bytes: u64,
}
```

Recommended eviction order for recorded entries:

1. Oldest `updated_at` first, across image and video namespaces.
2. Tie-break by canonical path and namespace for deterministic tests.
3. Delete database rows through existing writer/remove paths.
4. Remove recorded thumbnail files best-effort, matching current per-directory
   clear behavior.

The prune operation should also remove orphan thumbnails in arama's thumbnail
directory: files that live under the configured thumbnail directory but are not
referenced by any current image or video cache row. This is part of RFC 016
because orphan thumbnails count toward actual footprint and are rebuildable.

Important limitation: deleting SQLite rows may not shrink the database file on
disk immediately. The report must therefore show actual `before`/`after`
footprint measured from the filesystem, and thumbnail bytes removed must be
reported separately. `VACUUM` is out of scope unless `localcache` exposes a
safe maintenance hook.

If `max_bytes` is lower than the database and sidecar footprint that cannot be
reclaimed without SQLite compaction, pruning should stop once recorded
thumbnails and orphan thumbnails have been removed as needed. It must not delete
all remaining database-only entries merely to chase an unreachable target.
`target_reached` is `false` in that case, and `unreclaimable_bytes` records the
measured bytes that remain outside RFC 016's reclaimable scope.

### Part C — Cache page UX

Extend the Cache page with cache-footprint visibility and explicit pruning:

- Show actual cache footprint near the existing summary.
- Keep the existing per-directory table, but distinguish source media size from
  cache footprint in labels/docs.
- Add a one-off numeric prune target input or preset selector.
- Add a prune button that runs the explicit prune task.
- After pruning, reload the table and footprint.
- Surface prune failures through the existing app error/toast channel.
- If a prune target cannot be reached because only unreclaimable database bytes
  remain, show that in the prune result copy instead of presenting it as a
  failure.

The first implementation should not prune automatically on page load or after
indexing runs.

### Part D — Settings / persistence

Do not add persisted cache-capacity settings in the first implementation.

The first implementation uses a one-off Cache page prune target only. A
persisted setting such as `cache_limit_bytes: Option<u64>` should wait for a
later RFC that designs automatic pruning, repeated reminders, or durable
user-preference behavior.

### Part E — Disk-pressure warnings

Low-disk-space warnings are related but distinct from cache limits:

- Cache limit: "How large may arama's cache become?"
- Disk pressure: "Is the filesystem running out of free space?"

For this RFC, disk pressure should be informational:

- File System settings may show current free/total disk space as it does today.
- Cache page may show a warning when free space is low.
- No automatic deletion should happen solely because free space is low in the
  first implementation.

## Touches in detail

### `crates/cache/`

Add a maintenance module or equivalent public API for footprint measurement and
explicit pruning. The API should be namespace-aware internally but present a
single cache-level operation to callers.

### `crates/cache/src/types.rs`

Add public `CacheFootprint`, `CachePruneRequest`, and `CachePruneReport` types,
or equivalent names.

### `crates/ui/main/src/core/views/cache_page*`

Render cache footprint, one-off prune target input/control, explicit prune
action, and prune result copy. Keep table layout stable and avoid conflating
media size with cache footprint.

### `app/src/core/update/cache.rs`

Route prune requests through an async task and existing app-level error/toast
handling. Reload Cache page rows and footprint after pruning.

### `crates/i18n/src/en.rs` and `crates/i18n/src/ja.rs`

Add localized labels for footprint, prune target, prune action, success,
partial success / target-not-reached, failure, and low-disk warning text.

### `docs/src/users/cache.md`

Document the difference between indexed media size and actual cache footprint,
plus explicit prune behavior and target-not-reached reporting.

### `docs/src/dev/workspace.md`

Document the new cache maintenance API.

### `CHANGELOG.md`

Record cache footprint/pruning under `[Unreleased]`.

### `rfcs/README.md`

Add RFC 016 to the Proposed table while under review, then move it to
Implemented when shipped.

## Non-goals

- No persisted cache-limit setting in the first implementation.
- No automatic background eviction by default.
- No automatic pruning after indexing runs.
- No SQLite `VACUUM` unless a safe localcache-supported maintenance path exists.
- No change to thumbnail dimensions, embedding payload formats, or similarity
  scoring.
- No release action. Release timing remains owner-driven.

## Risks

- Pruning based on filesystem footprint may not immediately reduce database file
  size because SQLite can retain free pages. Mitigation: report measured
  before/after footprint, `target_reached`, and `unreclaimable_bytes`.
- Automatic eviction could surprise users. Mitigation: first implementation is
  explicit/manual.
- Cache page controls can become crowded. Mitigation: keep the footprint/prune
  controls compact and avoid duplicating Settings controls.
- Cross-namespace pruning must be deterministic. Mitigation: sort by
  `updated_at`, path, and namespace, and add regression tests.
- Orphan-thumbnail cleanup could remove files manually placed in arama's
  thumbnail directory. Mitigation: only operate inside the configured arama
  thumbnail directory, which is application-owned cache storage.

## Test plan

- Unit/integration tests for footprint measurement with missing files, database
  files, sidecars, and thumbnail directories.
- Cache prune tests proving oldest-first removal across image/video namespaces.
- Tests proving thumbnail deletion is best-effort and database row deletion is
  authoritative.
- Tests proving orphan thumbnails are included in footprint and cleaned during
  prune.
- Tests proving unreachable targets report `target_reached = false` and do not
  delete all remaining database-only entries.
- Existing cache integration tests remain green.
- Workspace gates:
  - `cargo fmt --all --check`
  - `cargo test -p arama-cache`
  - `cargo test --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo audit`
  - `mdbook build docs`

## Open questions

1. Should "low disk" use an absolute threshold, a percentage threshold, or only
   display free/total space without warnings?
2. Should `VACUUM` be considered later if localcache exposes a safe maintenance
   hook?
3. Should automatic pruning after indexing become a later RFC once explicit
   pruning is proven?
