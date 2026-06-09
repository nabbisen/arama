# Testing

## Test organisation

Test code is kept separate from implementation code:

| Pattern | Where |
|---|---|
| Unit tests for a module | `src/<module>/tests.rs` or inline `#[cfg(test)] mod tests` |
| Integration tests for a crate | `tests/integration_tests.rs` |
| When `tests.rs` grows large | Split into `tests/<category>.rs` submodules |

The same ELOC limits (300 / 500) apply to test files.

## Running tests

```sh
# All tests
cargo test --workspace

# One crate
cargo test -p arama-cache

# Specific test by name
cargo test -p arama-cache image_lookup_invalidated
```

## `arama-cache` integration tests

`crates/cache/tests/integration_tests.rs` is the **API compatibility
contract** for the cache facade. The test file was written against the
v1 (`file-feature-cache`) implementation and passes unchanged against
the v2 (`localcache`) implementation — this is the RFC 002
compatibility proof. Do not remove or weaken existing tests when
refactoring the cache internals.

Tests cover:
- Upsert and lookup (image and video)
- Invalidation when a file changes (different-length overwrite for
  deterministic size-based detection)
- COALESCE semantics: `None` vectors in an upsert preserve existing values
- Parallel lookup via cloned readers sharing the same read pool
- Partial failure in batch upsert (`upsert_all`)
- Thumbnail generation to a directory (`.jpg` suffix required for
  `image::open` format detection)
- Directory-scoped queries (`all_in_dir`, `all_in_dir_and_sub_dirs`)
- Persistence across writer/reader lifecycles

## `arama-ai` tests

`crates/ai/src/config/video_similarity_config.rs` has unit tests for
the timestamp computation logic (`compute_sample_timestamps`). These
are small and fast; they do not require model files.

## Testing with the UI

There are no automated UI tests. Manual verification steps for a
release build:

1. Fresh install: setup wizard downloads models and ffmpeg without error.
2. Select a directory with images → gallery populates; spinning
   indicators stop when indexing finishes.
3. Click a gallery image → focus view shows similar images sorted by
   score.
4. Switch directories mid-index → previous indexing stops, new one
   starts.
5. Settings → General: toggle media types → re-indexing triggers.
6. Settings → File system → Cache delete → confirm cache dir is
   removed.
7. Settings → AI: if models present, shows "ready"; if absent, shows
   Load/Get buttons.

## Avoiding regressions

Design specs (RFCs) are the source of truth for test design. When an
RFC implementation note records a behaviour change (e.g. "Invalidated
no longer deletes the stale row"), add a test comment explaining the
expected behaviour and why it differs from the original description.
