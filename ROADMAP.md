# Roadmap

This roadmap records near-term project themes that are specific enough to
guide planning but not yet implementation promises. Non-trivial themes move
through the RFC process before code changes begin.

Themes are agreed jointly by the project owner and the architect. The current
set was agreed on 2026-08-03, after 0.37.0 shipped and the RFC queue emptied.

## Current focus

### A — Release and CI reliability

**Status.** Agreed 2026-08-03. RFC 034 shipped in 0.38.0, whose tag push is
itself the mechanism's end-to-end verification. RFC 035 shipped in 0.38.0. The
sibling native-smoke RFC is still unwritten, and **cause 4 is still
unidentified** — see below.

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

**RFCs.**

- **RFC 034 — release workflow reliability.** *Implemented, verification
  pending.* Tag-push-triggered release creation removes causes 1–3
  structurally — the tag carries the workflow that will run, the trigger is
  `push`, and the ref is a tag by construction. All five variants must build
  before the release is created, so a failure leaves no release rather than a
  partial one. Plus `--clobber` for idempotent re-runs, asset-count
  verification, a manifest-must-equal-tag gate, and pre-tag build verification
  via `workflow_dispatch`. **Cause 4 is still unidentified**; the 0.38.0 cut is
  the first real test of whether it recurs.
  - Pre-release tag support was accepted, implemented, then withdrawn: internal
    dependencies carry `version = "0"` per RFC 030, and a Cargo requirement
    without a pre-release component never matches a pre-release version, so no
    pre-release could build. Recorded in the RFC rather than deleted.
- **RFC 035 — similarity-dialog cache-error routing.** *Implemented.* Closes
  RFC 033's Part B deferral: cache-read failures in the similar-pairs and focus
  dialogs no longer render as an empty result indistinguishable from "nothing
  found".
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

Three of the original four items have shipped. What remains, plus one successor:

- **~~Similarity-dialog cache-error tier routing (RFC 035).~~** *Shipped.*
  Closed RFC 033's Part B deferral. Cache-read failures in both similarity
  dialogs now surface as one inline message per dialog open, with partial
  results retained.
- **Similarity-dialog absence states
  ([RFC 036](./rfcs/proposed/036-similarity-dialog-absence-states.md)).**
  *Proposed, awaiting owner decision.* RFC 035 gave failure a voice and left two
  silences behind: a similar-pairs dialog with zero results renders **no text at
  all** — indistinguishable from still-loading — and a user missing ffmpeg gets
  image-only results with no in-dialog explanation. Both were found with
  rendered evidence during RFC 035's cycle. Neither is a regression; the RFC's
  own open question is whether it lands before or after 0.38.0.
- **ELOC remeasurement.** Nothing currently exceeds the 500-ELOC "strongly
  recommended" threshold. Measured 2026-08-03: `app/src/core.rs` raw 552 /
  ELOC ≈474, `app/src/core/update/cache.rs` 463, `app/src/core/update/ffmpeg.rs`
  450, `video_engine.rs` raw 355 / ELOC ≈307. `app/src/core.rs` is the one to
  watch — it will cross on its next material growth, and it grew during the
  0.37.0 cycle. Open a split RFC only after an exact measurement identifies a
  coherent scope.
- **~~`event-listener` 5.4.2.~~** *Shipped.* RUSTSEC-2026-0221 resolved by a
  straight bump; no ignore or override was needed. The audit warning count went
  from five to four.
- **~~`localcache` 0.21.1 / 0.21.2.~~** *Shipped.* Routine bump to 0.21.2. The
  `rusqlite`/`libsqlite3-sys` chain RFC 033 selected is unchanged and the cache
  suite, including the `ReadPool` poisoning tests, passes unmodified.

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

**0.38.0** shipped RFCs 034 and 035, plus a dependency-maintenance pass
(`app-json-settings` 2.5.1, `event-listener` 5.4.2, `localcache` 0.21.2). It is
the first release cut under tag-push-triggered creation, and the first since
0.37.0 to carry executable assets.

**0.37.0** (2026-08-03) shipped RFCs 015–030, 032, and 033, plus an
audit-warning burn-down maintenance pass. RFC 031 was archived as superseded by
RFC 032. See [`CHANGELOG.md`](./CHANGELOG.md) for the user-facing record and
[`rfcs/README.md`](./rfcs/README.md) for the RFC index.

One item from 0.37.0 remains open, tracked under theme A:

- archive and built-executable artifact-absence inspection ran and passed
  against the source tarball and the Linux binary, but no Windows or macOS
  executable existed to inspect. 0.38.0 produces them, so the inspection can
  finally run against every shipped target.

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
