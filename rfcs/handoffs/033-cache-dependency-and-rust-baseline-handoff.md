# RFC 033 Handoff — Cache dependency correction and Rust source-build baseline

Companion to [RFC 033](../proposed/033-cache-dependency-and-rust-baseline.md).
RFC 033 is **accepted for implementation** and remains in `rfcs/proposed/` until
the work ships, per the four-folder lifecycle in
[RFC 000](../done/000-rfc-lifecycle-policy.md). This handoff directs execution;
it does not change the RFC's lifecycle state and does not override its design.

## 1. Design authority and precedence

Check the implementation against, in order:

1. [RFC 033](../proposed/033-cache-dependency-and-rust-baseline.md) — the
   governing design for the dependency choice, the baseline declaration, and
   its CI verification;
2. [RFC 017 — Visible recoverable error UX](../done/017-visible-recoverable-error-ux.md)
   — the tier model that the new poisoning error must follow;
3. [RFC 002 — Replace the cache engine with localcache](../done/002-replace-cache-engine-with-localcache.md)
   — the cache facade API compatibility contract proven by the existing
   integration suite;
4. [RFC 023 — Cache serialization dependency strategy](../done/023-cache-serialization-dependency.md)
   — still the authority for cache payload/codec strategy, which this work does
   **not** touch.

Where they overlap, RFC 033 controls the dependency version and the baseline
value. RFC 017 controls how the new error reaches the user. RFC 023 controls
anything about payload encoding — meaning: leave it alone.

If implementation uncovers a conflict with RFC 033, stop and raise a design
request. Do not resolve it in the handoff or in code.

## 2. Implementation handoff

**Goal.** Move arama off the `localcache 0.20.0` dependency graph that cannot
build on its own declared baseline, adopt `0.21.0`, declare a verified Rust
1.91 contributor baseline in one place, and keep that declaration true with one
CI job.

### Decisions already made — do not re-open

These are settled in RFC 033. Re-deciding any of them in code is out of scope
and will be sent back:

1. **`localcache = "0.21"`, not `0.20.1`.** Both fix the toolchain break;
   0.21.0 is required because `arama-cache` uses `ReadPool` on a reachable
   panic path. Do not "simplify" this to a patch bump.
2. **No direct `rusqlite`, `libsqlite3-sys`, or SQLite dependency, and no
   `[patch]` or transitive pin.** Dependency ownership stays with `localcache`.
3. **Do not search downward for the true MSRV floor.** Verify 1.91. If it
   fails, step *up* to the next installed toolchain until one passes. A lower
   version passing is not a reason to lower the declaration.
4. **`Poisoned` is never a cache miss.** A miss asserts that nothing is cached;
   a poisoned pool means the data is unknown. Collapsing the two is a false
   statement to the user.
5. **The CI job pins an exact version**, e.g. `1.91`, never `stable` or another
   floating channel. A channel that drifts upward silently stops testing the
   claim, which is the whole defect being corrected.
6. **The 1.95 revert belongs to the RFC 032 tail commit**, not to the baseline
   commit. The baseline must change exactly once, in Task 3.

### Change scope

- `Cargo.toml` — `[workspace.dependencies] localcache`, and
  `[workspace.package] rust-version`
- `Cargo.lock` — refreshed, reviewed
- `crates/cache/src/core/engine.rs`, `image.rs`, `video.rs` — error surface only
- `crates/cache/src/core/image/tests.rs` — new crate-internal test module
  (and the `video` sibling if mirrored). **Amended 2026-08-01** after review
  059: the original `crates/cache/tests/**` scoping was wrong, because the
  poisoning path is not reachable from the public-API integration boundary.
- `.github/workflows/` — one new MSRV job
- `docs/src/users/installation.md`, `docs/src/dev/workflow.md` — baseline mirrors
- `CHANGELOG.md` `[Unreleased]`, `ROADMAP.md` milestone 1

### Explicit non-change scope

- cache schema, payload format, codec, or namespace (RFC 023 authority);
- `app/` and `crates/ui/*` error routing, including the similarity dialogs —
  deferred by RFC 033 Part B, confirmed at review 059;
- the `arama-cache` public facade API and existing test expectations;
- RFC 032's external-FFmpeg policy, and the 031/032 lifecycle move;
- `event-listener` / `cargo audit` ledger work (RFC 027 authority);
- any further CI jobs — format, Clippy, audit, feature matrix, cross-target;
- `CHANGELOG.md` shipped sections, including the `[0.34.0]` `RFC-033` wording;
- version bump, archive, tag, publish, commit-to-main, or release action.

### Relevant seams

| Concern | Location |
|---|---|
| Dependency requirement | `Cargo.toml` `[workspace.dependencies]` |
| Error wrap point | `crates/cache/src/core/engine.rs:49` — `Engine(#[from] localcache::LocalFileCacheError)` |
| `ReadPool` fields | `image.rs:43,236`; `video.rs:47,229` |
| `ReadPool::open` | `image.rs:57,247`; `video.rs:58,238` |
| Parallel read fan-out | `image.rs:278-286` (`lookup_all`) and the video equivalent |
| Consumers above it | `ui/widgets/.../similar_pairs_dialog.rs:231`, `.../media_focus_dialog/similar_media.rs:117,204`, `ai/src/pipeline/score/similarity/image.rs:25` |
| Error tier routing | `app/src/core/update/cache.rs`, cache page view, similarity dialogs |

### Non-obvious pitfalls

- **The facade wrapping is what makes `#[non_exhaustive]` free.** Nothing in the
  workspace matches `LocalFileCacheError` exhaustively. If you find yourself
  adding a `_` arm, check why you are matching it at all.
- **Verify the bundled SQLite version moved sensibly.** `libsqlite3-sys 0.37.0`
  may bundle an *older* SQLite than `0.38.x`. Record both; do not assume the
  newer transitive release was strictly safer.
- **Review the lockfile diff before accepting it.** Stop if dependencies
  unrelated to the SQLite chain move.
- **The poisoning test must use `read_conns = 1`.** `ReadPool::checkout` skips a
  poisoned slot during its `try_lock` scan exactly as it skips a busy one, and
  reports `Poisoned` only from the blocking fallback when no slot remains. A
  multi-slot pool therefore will not surface the error deterministically.
  Record that reason in the test so nobody later "simplifies" the size away.
- **Verify the panic actually poisons; do not assume it.** Run the test and
  report its observed output.
- **Do not weaken the assertion to whatever passes.** The property is
  `Err(..)` and specifically *not* `Ok(LookupResult::Miss)`. Match the variant
  with `matches!` — `LocalFileCacheError` is `#[non_exhaustive]`, so it can be
  matched with a `_` arm but not constructed from outside `localcache`.

## 3. Task breakdown / PR plan

Four independently reviewable units, in order.

**Task 1 — Land the RFC 032 tail.**
Commit the pre-existing uncommitted external-FFmpeg work (`scripts/`, sidecar
smoke test, `docs/src/dev/*`, `docs/src/dev/workspace.md`) **with the
provisional 1.95 edits reverted** from `Cargo.toml`, `CHANGELOG.md`, and
`docs/src/users/installation.md`. Committed source declares 1.90, unchanged,
at the end of this task. No RFC 033 work included.

**Task 2 — Adopt `localcache 0.21.0` and surface poisoning.**
Bump the requirement, refresh and review the lockfile, confirm
`libsqlite3-sys 0.37.0`, route the new error per RFC 017, add the focused test.
The bump and the error routing ship together — an intermediate state where
`Poisoned` is unhandled is not an acceptable review point.

**Task 3 — Declare and enforce the baseline.**
Verify 1.91 on the exact toolchain, set `[workspace.package] rust-version`,
mirror it in the two documentation locations, and land the CI job — all in one
change, so the job's first run validates the declaration it enforces.

**Task 4 — Record.**
`CHANGELOG.md` `[Unreleased]` and `ROADMAP.md` milestone 1. May combine with
Task 3 if that keeps the review coherent.

## 4. Acceptance and QA checklist

### Dependency

- [ ] `localcache = "0.21"` in `[workspace.dependencies]`; lockfile resolves
      `libsqlite3-sys 0.37.0`.
- [ ] No direct `rusqlite`/`libsqlite3-sys`/SQLite dependency and no `[patch]`
      or pin anywhere in the workspace.
- [ ] Lockfile diff contains no dependency churn unrelated to the SQLite chain.
- [ ] Bundled SQLite versions before and after are recorded.

### Error behaviour

- [ ] A poisoned read pool surfaces as an error, never as `Miss` and never
      silently.
- [ ] No `.ok()`, `.unwrap_or(Miss)`, or equivalent swallows a pool error on any
      read path in `image.rs` or `video.rs`.
- [ ] A focused crate-internal test at `read_conns = 1` proves it, and was
      actually executed.
- [ ] Similarity-dialog tier routing is **not** attempted here — deferred per
      RFC 033 Part B to a follow-up RFC. The Cache page already satisfies the
      blocking-view case with no change.

### Baseline

- [ ] `[workspace.package] rust-version` is 1.91, or higher with the dependency
      that required it recorded.
- [ ] Verified by an *executed* `rustup run <version>` command, not inferred
      from stable passing. Paste the output.
- [ ] Exactly one normative declaration; `installation.md` and `workflow.md`
      mirror it and changed in the same commit.
- [ ] The provisional 1.95 edits are absent from committed source.

### CI

- [ ] One job, pinned to the exact declared version, running
      `cargo check --workspace --locked` (plus tests where cost permits, with
      the reason recorded if omitted).
- [ ] Triggered on push and pull request against `main`.
- [ ] No other CI job added.
- [ ] `docs/src/dev/workflow.md`'s "no CI enforcement" statement updated, scoped
      to what the job actually covers.

### Regression

- [ ] Existing `arama-cache` image/video/cross/maintenance suites pass
      **unchanged**. They are the RFC 002 compatibility contract; weakening an
      assertion to make the suite pass is a blocking finding.
- [ ] `cargo fmt --all --check`, `cargo check --workspace --locked`,
      `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D
      warnings`, `cargo audit` — each reported only if observed.

### Boundaries

- [ ] No RFC 031/032 lifecycle movement.
- [ ] No version bump, archive, tag, publish, or release action.
- [ ] No cache payload/codec change.
- [ ] Shipped `CHANGELOG.md` sections untouched.

## 5. Required review-request content

Submit per the standard format:

1. Implementation summary;
2. Addressed RFC 033 requirements, by Part;
3. Changed files;
4. Important implementation decisions;
5. Any difference from RFC 033, stated explicitly — differences are reviewable,
   silent ones are not;
6. Executed commands and their **observed** output, including the exact
   toolchain run and the lockfile diff;
7. Test results;
8. Build, Clippy, and audit results;
9. Unresolved issues and design requests;
10. Known limitations;
11. Requested review focus.

Report a command that was not run as *not run*. Do not report a gate as clean
because it was clean before the change.
