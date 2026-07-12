# RFC 023 - Cache serialization dependency strategy

**Status.** Implemented (Unreleased)
**Tracks.** Remaining audit-warning owner follow-up: decide whether arama can
remove the `localcache` -> `bincode` 2.0.1 warning without destabilizing the
cache engine or persistent cache payload format.
**Touches.** `Cargo.toml`, `Cargo.lock`, `crates/cache/Cargo.toml`,
`crates/cache/src/core/engine.rs`, `crates/cache/src/core/payload.rs`,
`crates/cache/tests/**`, `rfcs/notes/audit-warning-burn-down.md`,
`CHANGELOG.md`.

## Summary

After RFC 022, `cargo audit` reports three allowed warnings. The remaining
`bincode` warning is owned by the cache engine:

```text
bincode 2.0.1 <- localcache 0.20.0 <- arama-cache
```

`cargo outdated --workspace --depth 1` reports all direct dependencies up to
date, and `cargo info localcache` reports 0.20.0 as the current crate version
in this environment. This is therefore not a normal dependency update.

This RFC proposes a design gate before touching cache storage:

1. Prefer a `localcache` codec/dependency solution that keeps arama on the
   existing cache engine and removes `bincode` from arama's dependency graph.
2. Treat a full cache-engine replacement as a fallback only if `localcache`
   cannot expose a bincode-free codec path.
3. Require cache compatibility and migration evidence before changing payload
   encoding for existing users.
4. Keep the remaining `paste` and `ttf-parser` warnings tracked separately.

## Why

The earlier audit-warning work intentionally avoided replacing `localcache`
because it owns arama's persistent cache behavior. Unlike `hnsw_rs`, this
dependency is not localized to one scoring function:

- image and video payloads are serialized through `localcache`;
- cache freshness, invalidation, read pools, summaries, delete/prune behavior,
  and thumbnail paths all sit behind the `arama-cache` facade;
- cache databases are user-local persistent files, so payload format changes
  can invalidate useful work or break first-run-after-upgrade behavior.

`localcache` 0.20.0 already has a `Codec::Json` behind its `json` feature, but
the crate still declares `bincode = "2"` unconditionally. Switching arama to
JSON in 0.20.0 would not remove the audit warning by itself. It would also
change payload size and compatibility behavior, so it must be justified by a
real dependency-graph outcome.

## Design

### Part A - Preferred path: upstream or patched bincode-free localcache codec

The preferred implementation path is:

1. Make or consume a `localcache` version where `bincode` is optional and not
   compiled when callers choose a non-bincode codec.
2. Enable the `json` codec only if `cargo tree -i bincode@2.0.1` confirms the
   warning is removed from arama.
3. Keep arama's `arama-cache` public facade stable.
4. Bump cache payload versions or namespace only if existing bincode payloads
   cannot be read safely after the codec change.

Acceptable variants:

- wait for an upstream `localcache` release that makes `bincode` optional;
- temporarily use a tightly scoped workspace patch to validate the upstream
  change before release consideration;
- retain `localcache` 0.20.0 if the bincode-free codec path is not available
  or not worth the compatibility tradeoff.

The implementation must not add a broad audit ignore for `bincode`.

### Part B - Compatibility contract

Before changing arama's cache codec, the implementation must define and test
the upgrade behavior for existing cache databases:

- existing bincode entries remain readable and are rewritten only as part of a
  deliberate migration; or
- existing bincode entries are treated as stale by a payload-version or
  namespace bump and are rebuilt naturally by later cache runs; or
- the implementation retains bincode until `localcache` can support mixed-codec
  read/write migration safely.

The selected behavior must be visible in the review package. Silent decode
failure, panics, or mixed payload corruption are not acceptable outcomes.

### Part C - Cache facade behavior to preserve

The implementation must preserve the current `arama-cache` contract:

- image and video upsert/lookup behavior;
- thumbnail path generation and best-effort thumbnail deletion;
- stale-file invalidation;
- `None` vector update semantics for video payloads;
- read-pool parallel lookup behavior;
- `all`, `all_in_dir`, `all_in_dir_and_sub_dirs`, `list_paths`, and
  `summarize_by_dir`;
- cache clear and prune behavior;
- parent directory creation and reader-first schema initialization.

### Part D - Fallback: cache-engine replacement

If `localcache` cannot provide a bincode-free path, replacing the cache engine
is a separate high-risk implementation and should come back for review before
code changes.

A replacement proposal must cover:

- schema shape and payload encoding;
- migration or explicit rebuild policy from the current localcache database;
- read/write concurrency model;
- freshness detection parity;
- cache page summary/prune/delete support;
- expected dependency graph and audit impact.

Do not replace the cache engine merely to reduce the warning count by one if it
creates a larger persistence risk.

### Part E - Audit note update

After implementation:

- remove or revise the `bincode` 2.0.1 entry only if `cargo audit` and
  `cargo tree -i bincode@2.0.1` confirm the warning is gone;
- otherwise record the chosen retention rationale;
- keep `paste` and `ttf-parser` as separate remaining owners.

## Touches in detail

### `Cargo.toml`, `crates/cache/Cargo.toml`, and `Cargo.lock`

Expected only if the selected implementation changes `localcache` features,
uses a workspace patch, consumes a newer `localcache`, or adds a new cache
codec/engine dependency. The review package must summarize lockfile churn and
show the `bincode` owner outcome.

### `crates/cache/src/core/engine.rs`

May need to set a non-default `localcache::Codec`, adjust payload version or
namespace handling, and document the selected cache-upgrade behavior.

### `crates/cache/src/core/payload.rs`

May need documentation updates if the payload wire format changes. Payload
shape should not change unless required; changing shape requires a version bump
and explicit tests.

### `crates/cache/tests/**`

Add or extend focused integration tests for any cache codec or migration
decision. Existing image/video/cross/maintenance tests remain the baseline.

### `rfcs/notes/audit-warning-burn-down.md`

Update only with observed dependency/audit results.

### `CHANGELOG.md`

Record the implementation as cache serialization/dependency strategy. If cache
entries are rebuilt after upgrade, state that plainly.

## Non-goals

- No release action or RFC lifecycle movement.
- No broad audit ignore.
- No model, setup, AI scoring, image similarity, or UI redesign.
- No cache page feature expansion.
- No migration of settings/config files.
- No forced cache-engine replacement if a lower-risk localcache path is viable.

## Risks

- JSON payloads are larger than bincode payloads. Mitigation: measure a sample
  image/video payload size and document the expected disk impact if JSON is
  selected.
- Existing bincode entries may not decode under a JSON-only writer. Mitigation:
  select an explicit migration, version-bump, or retention policy and test it.
- A workspace patch can hide unreleased upstream risk. Mitigation: use it only
  as validation evidence unless the owner explicitly accepts a patched release
  strategy.
- Replacing localcache could regress cache behavior. Mitigation: require the
  full cache integration suite plus a migration/rebuild story before adopting a
  replacement.

## Test plan

Required default gates:

```sh
cargo fmt --all --check
cargo check --workspace
cargo test -p arama-cache
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo audit
```

Dependency checks:

```sh
cargo tree -i localcache@0.20.0
cargo tree -i bincode@2.0.1
cargo tree -i paste@1.0.15
cargo tree -i ttf-parser@0.25.1
```

Focused cache behavior tests:

- image payload round-trip;
- video payload round-trip with partial vector updates;
- stale-file invalidation;
- reader-first schema initialization;
- parallel reader lookup;
- directory summaries and non-recursive delete;
- prune behavior and thumbnail best-effort deletion;
- selected codec/migration/rebuild behavior.

Size and migration evidence:

- compare serialized payload size for representative image and video payloads
  if switching away from bincode;
- test or manually demonstrate the selected existing-cache upgrade path;
- show whether `cargo audit` moves from three allowed warnings to two or
  remains at three with a documented retention rationale.
