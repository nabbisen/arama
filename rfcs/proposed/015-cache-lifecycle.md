# RFC 015 — Cache lifecycle: retire v1 migration and define cache-capacity direction

**Status.** Proposed
**Tracks.** RFC 002 follow-up: remove the temporary v1 cache migration shim
after the compatibility window, and decide whether disk-pressure/cache-size
management belongs in this RFC or a later one.
**Touches.** `crates/cache/`, `app/src/core.rs`, `Cargo.toml`,
`crates/cache/Cargo.toml`, `env/src/file.rs`, `CHANGELOG.md`,
`docs/src/users/cache.md`, `docs/src/users/installation.md`,
`docs/src/dev/workspace.md`,
`rfcs/README.md`.

## Summary

arama still carries the one-time v1 cache migration path introduced when
RFC 002 replaced the old `file-feature-cache` engine with `localcache`. That
migration shim was intentionally temporary: it imported a legacy v1 database
into the v2 cache database, renamed the v1 file to a backup on success, and was
scheduled for removal after one release cycle.

This RFC proposes retiring that migration path now, documenting the user-facing
recovery behavior, and deciding whether cache-size/disk-pressure management is
part of this work or a separate follow-up.

## Why

The temporary migration path has outlived its stated compatibility window.
Keeping it indefinitely has costs:

- `arama-cache` keeps legacy migration code that normal current users should no
  longer need.
- The old database reader keeps `rusqlite` relevant to the cache crate even
  though the active cache backend is `localcache`.
- Startup still has a historical migration branch that complicates reasoning
  about cache initialization and error recovery.
- The environment crate still exposes the legacy v1 cache filename and path
  helper solely so startup can find the old database for migration.
- Current user and developer docs still describe automatic v1 migration; those
  claims need to be removed when the migration path is removed.
- RFC 002 explicitly left disk-pressure/cache-size management as a follow-up;
  this is the natural point to either include it or record that it remains
  separate.

Removing migration must still be done carefully because caches are user data,
even if they are rebuildable. The design should prefer explicit documentation
and predictable recovery over silent behavior changes.

## Design

### Part A — Retire v1 migration

Remove the v1-to-v2 migration shim from `arama-cache` and remove the app startup
call that invokes it.

Expected behavior after removal:

- If a v2 cache database exists, arama uses it normally.
- If no v2 cache database exists, arama creates a fresh v2 cache lazily through
  the existing cache writers.
- A leftover v1 database is ignored rather than imported.
- A leftover `.v1.bak` file is ignored.

Documentation should explain that old cache files can be deleted if the user
wants to reclaim space. Cached thumbnails and embeddings are rebuildable.

### Part B — Dependency cleanup

If `rusqlite` is only needed for the retired migration shim inside
`arama-cache`, remove it from that crate after the shim is deleted. If another
tracked use remains, document the remaining owner and keep the dependency.

### Part C — Tests

Remove migration-specific tests that only validate the deleted v1 importer.
Keep or add tests that prove current cache initialization still creates and uses
v2 cache files normally.

Suggested checks:

- `cargo fmt --all --check`
- `cargo check --workspace`
- `cargo test -p arama-cache`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo audit`

### Part D — Cache-size and disk-pressure direction

RFC 002 named disk-pressure/cache-size management as a follow-up. This RFC
should choose one of two paths before implementation:

1. **Split path, recommended:** this RFC only removes the legacy migration path.
   A later RFC designs cache-size limits, eviction policy, and UI controls.
2. **Combined path:** this RFC also designs cache-capacity management.

The split path is recommended because migration removal is a bounded
compatibility cleanup, while cache-capacity management is product behavior with
UI, policy, and testing implications.

## Touches in detail

### `crates/cache/src/core/migrate.rs`

Delete the legacy migration module if no current code needs it.

### `crates/cache/src/core.rs`

Remove the `migrate` module declaration once the migration module is deleted.

### `crates/cache/src/lib.rs`

Remove the crate-level "Migrating from the v1 cache" docs and the public
re-export of `MigrationReport` / `migrate_v1_if_present`.

### `crates/cache/src/core/engine.rs`

Remove or update `CacheError::Migration` if no remaining code constructs it.

### `app/src/core.rs`

Remove the startup migration call. Startup should continue to initialize the
active cache normally and should not fail merely because an old v1 file exists.

### `env/src/file.rs`

Remove `CACHE_STORAGE_FILE_V1` and `cache_storage_path_v1()` if no
non-migration code still uses them. If the implementation deliberately keeps
either symbol, document the retained owner and purpose in this RFC's
implementation notes before moving it to `done/`.

### `crates/cache/Cargo.toml` and root dependency declarations

Remove `rusqlite` from `arama-cache` if the migration shim was its only use.
Keep workspace dependency definitions only if another crate still needs them.

### `docs/src/users/cache.md`

Document that current arama versions use the v2 cache and that old v1 cache
files are not imported anymore. Explain that cache files are rebuildable.

### `docs/src/users/installation.md`

Remove the current claim that a previous `.arama-cache/` directory is migrated
from the v1 cache format on first launch. Replace it with current behavior:
existing v2 cache files are reused, old v1 cache files are ignored, and cache
data can be rebuilt.

### `docs/src/dev/workspace.md`

Remove `migrate_v1_if_present` from the current `arama-cache` API summary and
update any workspace notes that still describe v1 migration as active behavior.

### `CHANGELOG.md`

Record the migration removal and any dependency cleanup under `[Unreleased]`.

### `rfcs/README.md`

Add RFC 015 to the Proposed table while under review, then move it to
Implemented when shipped.

## Non-goals

- No visible cache-management UI in this RFC if the split path is chosen.
- No new cache eviction policy in this RFC if the split path is chosen.
- No change to thumbnail dimensions, embedding payload formats, or similarity
  scoring.
- No release action. Release timing remains owner-driven.

## Risks

- A user who skipped many versions and still has only a v1 cache database will
  lose automatic cache import. Mitigation: cache data is rebuildable, and docs
  should explain the behavior.
- Removing `rusqlite` may affect transitive dependency shape. Mitigation:
  verify with `cargo tree` and normal gates.
- Removing public migration exports is an API change for direct `arama-cache`
  consumers. Mitigation: record it in the changelog and keep the user-facing
  application behavior clear.
- Removing `cache_storage_path_v1()` is an API change for direct `env` crate
  consumers. Mitigation: record the compatibility cleanup in the changelog and
  verify no workspace code still imports it.
- If app startup currently depends on migration side effects, removal could
  reveal an initialization assumption. Mitigation: add or keep cache startup
  tests focused on fresh v2 cache creation.

## Open questions

1. Should old v1 files be ignored silently, logged once, or mentioned in the
   Cache page if detected?
2. Is removal acceptable in the next patch/minor release, or should it wait for
   a named compatibility release?
3. Should cache-size/disk-pressure management be split into RFC 016?
4. Should docs include exact legacy filenames/locations, or keep the guidance
   higher level?
