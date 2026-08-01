# RFC 033: Cache dependency correction and Rust source-build baseline

**Status.** Proposed — accepted for implementation by the project owner
2026-08-01. Remains in `rfcs/proposed/` until the work ships, per RFC 000.
Execution is directed by
[the RFC 033 handoff](../handoffs/033-cache-dependency-and-rust-baseline-handoff.md).
**Tracks.** Correct the `localcache`-driven toolchain break, adopt the fixed
upstream release, and declare one measured source-build Rust baseline as a
single authority.
**Touches.** `Cargo.toml`, `Cargo.lock`, `crates/cache/src/core/engine.rs`,
`crates/cache/src/core/image.rs`, `crates/cache/src/core/video.rs`,
`crates/cache/tests/**`, `.github/workflows/` (one new MSRV job),
`docs/src/users/installation.md`, `docs/src/dev/workflow.md`, `CHANGELOG.md`,
`ROADMAP.md`, `rfcs/README.md`, `rfcs/notes/snora-recipe-theme-custom.md`.

## Summary

`localcache 0.20.0` declared `rust-version = "1.85"` while requiring
`rusqlite = "0.40"`, whose graph reaches `libsqlite3-sys 0.38.x` and the Rust
1.95 `cfg_select!` macro. Arama's locked graph therefore could not build on the
owner's preferred 1.91 baseline, and a provisional Rust 1.95 declaration was
staged in the working tree as a placeholder.

The upstream defect is fixed. `localcache 0.20.1` and `0.21.0` both constrain
`rusqlite` to `^0.39`, resolving `libsqlite3-sys 0.37.0`, which has no 1.95
requirement. The 1.95 constraint is gone and was never an arama requirement.

This RFC decides three things:

1. Arama adopts **`localcache 0.21.0`**, not `0.20.1`, because `arama-cache`
   uses `ReadPool` on a reachable panic path and only `0.21.0` reports a
   poisoned pool instead of silently returning abandoned state.
2. Arama declares **Rust 1.91** as a contributor-setup baseline in one
   authoritative field, **verified on that exact toolchain** and kept true by a
   CI job. It is not a minimized library compatibility floor, is not inferred
   from a newer compiler passing, and is not carried forward from a placeholder.
3. The provisional Rust 1.95 manifest, changelog, and installation edits are
   **discarded**, not committed.

## Why now

This is roadmap milestone 1 ("Source-build baseline decision") under the
external-FFmpeg closeout theme. It is a prerequisite for the RFC 032
implementation-closeout review, which cannot record a truthful MSRV or
cross-target gate while the declared baseline is a placeholder.

It is deliberately a separate RFC rather than closeout scope. RFC 032's
non-goals exclude dependency and version work, and folding a dependency bump
into its acceptance evidence would mean a reviewer accepting the external-FFmpeg
contract also implicitly accepts a cache-engine upgrade. Keeping them apart
preserves one theme per RFC and one authority per decision.

## Evidence

Verified against the crates.io index on 2026-08-01, independently of the
upstream maintainers' report:

| localcache | `rusqlite` requirement | resolves `libsqlite3-sys` | declared `rust-version` |
|---|---|---|---|
| 0.19.1 | `^0.40` | 0.38.x | 1.85 |
| 0.20.0 (current) | `^0.40` | 0.38.x | 1.85 |
| 0.20.1 | `^0.39` | 0.37.0 | 1.85 |
| **0.21.0** | `^0.39` | **0.37.0** | 1.85 |

Neither `rusqlite 0.40.1` nor `libsqlite3-sys 0.38.1` declares a
`rust-version`, so MSRV-aware resolution had nothing to fall back on. This
class of break is invisible to the resolver and can only be caught by compiling
on the exact declared toolchain — which is why this RFC requires that check
rather than a manifest edit alone.

Arama's current locked graph is `localcache 0.20.0 -> rusqlite 0.40.1 ->
libsqlite3-sys 0.38.1`, matching the broken row.

`localcache 0.19.1` carries the same defect. Upstream recorded the affected set
as "0.20.0 only"; this should be reported back, but it does not affect arama.

## Decision

### Part A — Adopt `localcache 0.21.0`

Both fixed releases resolve the toolchain break identically. The choice turns
on whether `arama-cache` uses `ReadPool`, which it does, extensively:

- `crates/cache/src/core/image.rs:43,236` and `video.rs:47,229` hold
  `ReadPool<ImagePayload>` / `ReadPool<VideoPayload>` fields on both the reader
  and the writer, in both namespaces;
- four `ReadPool::open` call sites at `image.rs:57,247` and `video.rs:58,238`.

The hazard is reachable rather than theoretical. `lookup_all`
(`image.rs:278-286`, and the video equivalent) fans out over the read pool with
rayon, and its consumers — `similar_pairs_dialog.rs:231`,
`media_focus_dialog/similar_media.rs:117,204`,
`ai/pipeline/score/similarity/image.rs:25` — run their own rayon fan-out above
it. A panic in any rayon worker holding a pool slot is exactly the poisoning
case.

On `0.20.1`, the next read silently recovers the poisoned guard and returns
state a panicking thread abandoned mid-operation. In arama that state becomes
cache hits feeding similarity scores, so the application would present a
similarity judgement derived from abandoned data with no signal at any layer.
For a tool whose entire product is that judgement, a silent wrong answer is a
worse failure than a visible error. `0.21.0` returns
`Poisoned { resource: "ReadPool" }` instead.

Arama must not add a direct `rusqlite`, `libsqlite3-sys`, or SQLite dependency,
and must not pin the transitive graph locally. Dependency ownership stays with
`localcache`.

### Part B — Surface pool poisoning truthfully

`0.21.0` makes a previously silent condition into a returned error, so the new
path must reach the user under the RFC 017 tiers rather than being swallowed:

- a poisoned pool during a cache read is a **blocking view error** where stale
  or unavailable data is being rendered (Cache page, gallery population);
- it is a **recoverable action error** where the user triggered a discrete
  action (similar-pairs, media focus);
- it must not be downgraded to a developer diagnostic, because the fallback
  would not be truthful — the data is unknown, not merely unavailable.

The implementation must not convert `Poisoned` into a cache miss. A miss is a
statement that nothing is cached; that is a different and false claim.

### Part C — Declare a verified contributor baseline, not a minimized floor

**Owner decision, 2026-08-01.** Arama is a GUI application, not a library.
No package depends on the workspace crates as libraries, so the declared
`rust-version` is not functioning as a downstream compatibility contract. Its
purpose is narrower and concrete:

1. it tells a contributor which toolchain to install to work on arama; and
2. it is the version whose build stability is verified before release.

The declaration is therefore a **chosen, clearly installable version**, not the
lowest technically possible one. Arama does not spend effort minimizing the
floor, and does not treat a later increase as a compatibility break requiring
migration ceremony.

The declared value is **1.91**, the owner's stated contributor baseline.

One requirement is retained without exception: **the declared version must be
verified by an exact-toolchain run, never inferred from a newer compiler
passing.** Declaring a version the graph cannot actually meet is precisely the
`localcache 0.20.0` defect this RFC exists to correct, and it is no less wrong
for being an application rather than a library.

If 1.91 does not pass, raise the declaration to the lowest tested version that
does, and record which dependency required it. No bisection of the true floor
is required or wanted.

For completeness of the factual record: `arama` and the member crates *are*
published on crates.io (0.36.2), so `rust-version` is a real registry claim.
This does not change the decision — no package depends on them, and
`cargo install` users can update their toolchain — but the declaration should
be truthful for that reason too, which Part F enforces.

### Part D — Discard the provisional 1.95 edits

The uncommitted working-tree edits declaring Rust 1.95 — in `Cargo.toml`,
the `CHANGELOG.md` "Rust build baseline" entry, and
`docs/src/users/installation.md` — are placeholders for an upstream constraint
that no longer exists. They are reverted, not committed.

Committed source continues to declare 1.90 until this RFC's measured value
replaces it in one change. The baseline must never be in two states at once.

### Part E — One authority for the baseline

To keep a single source of truth:

- `[workspace.package].rust-version` in the root `Cargo.toml` is the **only**
  normative declaration; all members inherit it and none restate it;
- `docs/src/users/installation.md` and `docs/src/dev/workflow.md` **mirror** it
  in prose and must be updated in the same change that alters it;
- this RFC records the decision and its rationale;
- `CHANGELOG.md` records the user-visible effect once, under the release that
  ships it.

No other file may assert a Rust baseline.

### Part F — Verify the baseline in CI

The declaration is kept true by automation rather than by memory. Arama adds
one CI job, scoped to this contract only:

- installs the exact declared toolchain;
- runs `cargo check --workspace --locked` on it;
- runs `cargo test --workspace` on it where runtime cost permits, otherwise
  check-only with the reason recorded;
- fails the job when the declared version cannot build the locked graph.

Trigger is push and pull request against `main`. The job must pin the exact
declared version rather than a channel alias, because a channel that floats
upward would silently stop testing the claim.

This is the direct lesson of the incident that produced this RFC: a declared
baseline with no automated check is a claim that decays without anyone
noticing. It would have caught `localcache 0.20.0` at its own release.

`docs/src/dev/workflow.md` currently states that there is no CI enforcement.
That statement is updated in the same change, scoped to what the new job
actually covers.

**Explicit non-goal:** this is not a general CI buildout. Format, Clippy,
`cargo audit`, feature-matrix, and cross-target jobs remain out of scope and
are a separate theme for a future RFC. Adding one narrow job here must not
become an unreviewed CI expansion.

### Part G — Disambiguate the "RFC-033" label

`rfcs/notes/snora-recipe-theme-custom.md` and its `rfcs/README.md` index row
describe the note as an "RFC-033 recipe". That is snora's RFC number used as a
document-format label, not an arama lifecycle number, and arama 033 is free.
Both live references are reworded to name snora explicitly so a future reader
grepping `033` is not led to two unrelated documents.

`CHANGELOG.md` line 267 carries the same wording inside the shipped `[0.34.0]`
section. Published release history is not rewritten; it is left as-is.

## Non-goals

- No direct SQLite, `rusqlite`, or `libsqlite3-sys` dependency, and no local
  transitive pin.
- No cache schema, payload format, codec, or namespace change. RFC 023 remains
  the authority for cache serialization strategy, and its decision to retain
  the current bincode-backed path is unchanged by this RFC.
- No change to RFC 032's external-FFmpeg policy, and no RFC lifecycle movement
  for 031 or 032.
- No `event-listener` / audit-ledger work. Arama's locked `event-listener 5.4.1`
  arrives via `zbus` and the `async-*` desktop-portal stack behind `rfd` and
  `file-handle`, not via `localcache`; it is tracked separately under the
  RFC 027 ledger.
- No release, version bump, archive, tag, publish, commit, or push action.
- No broad audit ignore.
- No general CI buildout. Part F adds exactly one MSRV job; format, Clippy,
  audit, feature-matrix, and cross-target automation remain a separate theme.

## Compatibility and migration

- `0.20.0 -> 0.21.0` requires no arama source change for the error surface: the
  workspace never matches `LocalFileCacheError` exhaustively. Its only
  reference is the wrapping variant at `crates/cache/src/core/engine.rs:49`,
  `Engine(#[from] localcache::LocalFileCacheError)`. The `#[non_exhaustive]`
  break costs nothing here.
- The two re-classified variants (`Poisoned` and `Serialization`, previously
  `UnsupportedFeature`) are not branched on anywhere in `app/`, `crates/`, or
  `env/`. Verified by identifier search.
- No cache database migration. Existing image and video cache entries remain
  readable; the change is dependency-level, not payload-level.
- Users of published executable assets are unaffected by the Rust baseline.
  Only source-build and `cargo install` routes are.

## Security considerations

This RFC narrows one integrity gap and opens none.

- Silent recovery of a poisoned read-pool guard is an **integrity** failure: it
  can return abandoned state as authoritative cache data feeding similarity
  results. Part A removes it; Part B makes the replacement visible rather than
  swallowed.
- Removing `libsqlite3-sys 0.38.x` in favour of `0.37.0` is a version movement
  inside a bundled-SQLite dependency. The implementation must confirm the
  resulting bundled SQLite version and record it, rather than assuming a newer
  transitive release is strictly safer.
- No trust boundary, network path, or executable-acquisition surface changes.

## Risks

- **1.91 does not pass.** Mitigation: raise the declaration to the lowest tested
  version that does and record the dependency that required it. This is a
  routine adjustment under Part C, not an escalation, because the baseline is
  not a compatibility contract.
- **The CI job is added but the declaration is still wrong at merge time.**
  Mitigation: the declaring change and the job land together, so the first run
  of the job validates the declaration it enforces.
- **`libsqlite3-sys 0.37.0` bundles an older SQLite than 0.38.x.** Mitigation:
  record both bundled versions and run the full `arama-cache` integration suite
  before accepting.
- **New `Poisoned` errors surface on paths that previously appeared to
  succeed.** This is the intended effect, but it can look like a regression.
  Mitigation: Part B's tier assignment plus a focused test, and a plain
  changelog statement.
- **Lockfile churn beyond the intended change.** Mitigation: review
  `git diff -- Cargo.toml Cargo.lock` and stop if unrelated dependencies move.

## Test plan and required evidence

Default gates on the chosen baseline and on stable:

```sh
cargo fmt --all --check
cargo check --workspace --locked
cargo test -p arama-cache
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo audit
```

Dependency verification:

```sh
cargo tree -i localcache
cargo tree -i libsqlite3-sys
cargo tree -i rusqlite
grep -A1 'name = "libsqlite3-sys"' Cargo.lock    # expect 0.37.0
```

Baseline verification on the exact declared toolchain:

```sh
rustup run 1.91 cargo check --workspace --locked
rustup run 1.91 cargo test --workspace
```

If either fails, retry on successively higher installed toolchains until one
passes, declare that version, and record which dependency required it. Do not
search downward for the true floor — a lower version passing is not a reason to
lower the declaration.

Focused cache evidence:

- the full existing image/video/cross/maintenance integration suite passes
  unchanged — it is the RFC 002 compatibility contract and must not be weakened;
- a focused test that a poisoned read pool surfaces as an error rather than a
  miss or a silent success.

Report only observed output. A toolchain that was not run is recorded as not
run.

## Acceptance criteria

- `localcache 0.21.0` is adopted; the locked graph resolves
  `libsqlite3-sys 0.37.0`; no direct SQLite dependency or transitive pin exists.
- Read-pool poisoning surfaces as a truthful user-visible error under the
  RFC 017 tiers and is never reported as a cache miss.
- The declared `rust-version` is 1.91, or a higher version with the dependency
  that required it recorded, and was verified by an exact-toolchain run in the
  review package rather than inferred.
- A CI job pins that exact version, builds the locked workspace, and lands in
  the same change as the declaration.
- The provisional 1.95 edits are absent from committed source.
- Exactly one normative baseline declaration exists; docs mirror it and were
  updated in the same change.
- The `RFC-033` label ambiguity is resolved in both live references, and shipped
  changelog history is untouched.
- The existing cache integration suite passes unchanged.
- No RFC 032 lifecycle, release, version, tag, or publish action is included.

## Alternatives rejected

**Take `localcache 0.20.1`.** Rejected. It fixes the toolchain break
identically but leaves the `ReadPool` poison-recovery hazard on a reachable
path, and defers the `#[non_exhaustive]` break to a less convenient moment
chosen by upstream rather than by arama.

**Keep `0.20.0` and declare Rust 1.95.** Rejected. It carries a baseline arama
does not need on behalf of an upstream defect that is already fixed, and raises
the source-build floor for contributors for no product reason.

**Pin `rusqlite` or `libsqlite3-sys` in arama.** Rejected, and upstream agrees.
It would move dependency ownership to the consumer and hide the real
requirement from anyone reading `localcache`'s manifest.

**Fold this into the RFC 032 closeout.** Rejected. It mixes two unrelated
design decisions into one acceptance gate and leaves the baseline contract
without an authoritative record.

## Resolved questions

1. **Baseline value and purpose.** Resolved by the owner, 2026-08-01: arama is
   an application, so the declaration exists for contributor environment setup
   and release stability, not as a minimized library floor. Declare 1.91 and
   verify it in CI. Recorded in Parts C and F. No floor bisection.
2. **`localcache 0.19.1` disposition.** Resolved by the owner, 2026-08-01:
   report the finding to the localcache project and let that project choose the
   remedy. Tracked outside this RFC as project communication; the finding itself
   stays recorded in the Evidence section above.

## Open questions

1. Should `0.21.0`'s `#[non_exhaustive]` prompt arama to branch on any
   `localcache` error variant in future? Not needed now; the facade wraps at
   `crates/cache/src/core/engine.rs:49`. Architect judgement, not an owner
   decision.

## Implementation sequence

1. Land the RFC 032 tail as its own commit, reverting the provisional 1.95
   edits (Part D) so committed source declares 1.90 unchanged.
2. Bump `localcache` to `0.21` in `[workspace.dependencies]`; refresh and
   review the lockfile; confirm `libsqlite3-sys 0.37.0`.
3. Run the cache suite; add the poisoning-surface test and route the error per
   Part B.
4. Verify 1.91 on the exact toolchain per Part C and record the result.
5. Declare the baseline in `[workspace.package].rust-version`, mirror it in the
   two documentation locations, and land the CI job, all in the same change
   (Parts E and F).
6. Update `CHANGELOG.md`, and record completion against `ROADMAP.md`
   milestone 1 — the milestone's framing was already corrected at acceptance,
   so this step records the result, not the decision. Request implementation
   review.
7. Only then proceed to the RFC 032 implementation-closeout review.

Part G lands with this RFC's own commit rather than at implementation, because
the ambiguity exists as soon as the number is claimed.
