# Roadmap

This roadmap records near-term project themes that are specific enough to
guide planning but not yet implementation promises. Non-trivial themes move
through the RFC process before code changes begin.

Themes are agreed jointly by the project owner and the architect. The current
set was agreed on 2026-08-03, after 0.37.0 shipped and the RFC queue emptied.

## Current focus

### A — Release and CI reliability

**Status.** Agreed 2026-08-03. RFC 034 to be proposed.

**Why now.** 0.37.0 released with its source archive and **no executable
assets, and no workflow run queued at all**. The cause is still unexplained:
the workflow is active, its trigger is correct on the default branch, and
GitHub registered two `release … published` events — yet no run started, on
either attempt.

That incident exposed four independent ways the release channel can silently
produce nothing, each indistinguishable to the operator from success:

1. the trigger listened for `created`, which does not fire for a release
   created already-published;
2. a trigger change may not be live the instant its push completes;
3. `workflow_dispatch` resolves the workflow from the *dispatched ref*, so a
   fallback added after a tag exists cannot rescue that tag's release;
4. one further cause, still unidentified.

Items 1 and 3 were fixed during the release itself (commit `8950240`:
`types: [published]`, plus `workflow_dispatch`). Those are **shipped**, and
RFC 034 documents rather than re-proposes them.

This theme is first because the release channel is the mechanism by which all
other work reaches users, and it is currently the least trustworthy part of the
project.

**Planned RFCs.**

- **RFC 034 — release workflow reliability.** Primary recommendation:
  tag-push-triggered release creation, which removes causes 1–3 structurally —
  the tag carries the workflow that will run, the trigger is `push`, and the
  ref is a tag by construction. Plus `--clobber` on the upload step so re-runs
  are idempotent, and pre-release build verification via `workflow_dispatch`
  against a branch.
- **A sibling RFC — native smoke on CI runners** (previously "route B").
  Extends automated native smoke to the `windows-latest` and `macos-latest`
  runners the release workflow already uses, covering the two highest-value
  unverified targets without owning hardware. Supersedes the corresponding rows
  in [`rfcs/notes/native-smoke-risk-acceptance.md`](./rfcs/notes/native-smoke-risk-acceptance.md)
  once it lands. It cannot cover Finder-launch `PATH` inheritance or rendered
  UI, which stay desktop-only.

**Evidence this theme is worth its cost.** Four defects were found during the
0.37.0 cycle — a release-blocking startup hang, a missing required action, a
prohibited affordance, and a false first-run error. **None was found by
automated gates**, which were green throughout the period all four existed.
Three came from rendered smoke; the fourth from the owner running the
application independently.

## Next

### B — Quality debt

**Status.** Agreed 2026-08-03. Follows theme A.

Four scoped items. Only the first needs an RFC.

- **Similarity-dialog cache-error tier routing (RFC 035).** Deferred from
  [RFC 033](./rfcs/done/033-cache-dependency-and-rust-baseline.md) Part B and
  recorded in its Status field. The similarity dialogs route **every**
  `CacheError` variant to `eprintln!` and render a partial or empty result. The
  gap predates RFC 033 and is variant-agnostic, so the RFC classifies all
  variants under [RFC 017](./rfcs/done/017-visible-recoverable-error-ux.md)
  rather than only the `Poisoned` case introduced there. Requires UX decisions:
  partial-versus-empty semantics, toast versus inline placement, new localized
  strings.
- **ELOC remeasurement.** Nothing currently exceeds the 500-ELOC "strongly
  recommended" threshold. Measured 2026-08-03: `app/src/core.rs` raw 552 /
  ELOC ≈474, `app/src/core/update/cache.rs` 463, `app/src/core/update/ffmpeg.rs`
  450, `video_engine.rs` raw 355 / ELOC ≈307. `app/src/core.rs` is the one to
  watch — it will cross on its next material growth, and it grew during the
  0.37.0 cycle. Open a split RFC only after an exact measurement identifies a
  coherent scope.
- **`event-listener` 5.4.2.** The locked graph carries 5.4.1 with
  RUSTSEC-2026-0221, reaching arama via `zbus` and the `async-*` desktop-portal
  stack behind `rfd`/`file-handle` — not via `localcache`. It is **not** in the
  [audit-warning ledger](./rfcs/notes/audit-warning-burn-down.md), which records
  four entries; the advisory postdates that refresh. A `cargo update -p
  event-listener` may resolve it outright.
- **`localcache` 0.21.1 / 0.21.2.** 0.21.0 is current in the lockfile and was
  chosen deliberately under RFC 033. 0.21.1 is published and unaffected by the
  MSRV defect; 0.21.2 is expected to carry the upstream `dependency_security`
  documentation of the affected version set (0.19.1 and 0.20.0). Assess as a
  routine bump, not a correction.

### C — Product direction

**Status.** Agreed as a theme 2026-08-03; **awaiting a problem statement from
the project owner.**

Every other candidate on this roadmap is infrastructure, quality, or debt.
arama has had no product theme proposed since the RFCs that built the current
UI. This theme exists to correct that, and it is deliberately empty until the
owner states what an arama user should be able to do that they currently
cannot.

The architect does not populate this section. Requirements, options, and an RFC
portfolio follow from the owner's problem statement, not from the architect's
inference about what users might want.

## Shipped

**0.37.0** (2026-08-03) shipped RFCs 015–030, 032, and 033, plus an
audit-warning burn-down maintenance pass. RFC 031 was archived as superseded by
RFC 032. See [`CHANGELOG.md`](./CHANGELOG.md) for the user-facing record and
[`rfcs/README.md`](./rfcs/README.md) for the RFC index.

Two items from that release remain open and are tracked under theme A:

- executable assets were never produced for 0.37.0;
- archive and built-executable artifact-absence inspection ran and passed
  against the source tarball and the Linux binary, but no Windows or macOS
  executable exists to inspect.

## Later candidates

### Remaining audit-warning owners

The remaining `bincode`, Candle/transitive `paste`, and font/rendering stack
warnings should be revisited when upstream releases expose compatible fixes or
when a replacement design is intentionally proposed.

### Release cadence

Release prep remains owner-driven. This roadmap does not set a release point;
it only identifies when a coherent reviewed batch may be ready for release
consideration. crates.io publication was deliberately deferred at 0.37.0 —
34 total downloads across two published versions made the channel's staleness
a low-impact trade — and is expected to resume at a future stable cut.
