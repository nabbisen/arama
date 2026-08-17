# Roadmap

This roadmap records near-term project themes that are specific enough to
guide planning but not yet implementation promises. Non-trivial themes move
through the RFC process before code changes begin.

Themes are agreed jointly by the project owner and the architect. The current
set was agreed on 2026-08-03, after 0.37.0 shipped and the RFC queue emptied.

## Current focus

### A — Release and CI reliability

**Status.** RFCs 034 and 035 shipped in 0.38.0. **Cause 4 did not recur** — the
tag-push trigger fired on all four pushes of that cut, retiring the unexplained
failure mode this theme was built around. The channel now produces executable
assets, verified against the published artifacts.

**What the cut cost:** four tag pushes and three defects, none of which was
visible on inspection — an EPIPE in the archive layout check, a latent EPIPE in
the notes pipeline, and a step-order bug that published a `0.38.0` release with
zero assets before it was caught and recovered.

**Closed in 0.39.0:** [RFC 037](./rfcs/done/037-release-publication-atomicity.md)
extended the guarantee inside the `release` job — the release is created as a
draft and published only after every asset is attached and counted — and the
notes-detection pipeline's silent-fallback defect was fixed alongside it.

**Closed in 0.39.1:**
[RFC 038](./rfcs/done/038-native-smoke-on-ci-runners.md) — native smoke now runs
on the `windows-latest` and `macos-latest` runners the release workflow already
pays for. Of the four residual risks named in
[`native-smoke-risk-acceptance`](./rfcs/notes/native-smoke-risk-acceptance.md),
two are fully discharged by execution — including the macOS native-prefix
fallback, the highest user-visible risk on the list, which had never run
anywhere — one is discharged for the Selected-directory path only, and
**Windows process-tree reaping on a probe timeout remains open**: neither smoke
variant produces a timeout, so it is recorded as undischarged rather than
implied by a green run.

Running it also surfaced two real defects on Windows —
[RFC 039](./rfcs/done/039-windows-path-search-reachability.md) and the
unreachable "install ffmpeg" message — neither of which any amount of reading
had found.

**Remaining in this theme:** Windows process-tree reaping, above. Nothing else.

**Why this theme existed.** 0.37.0 released with its source archive and **no
executable assets, and no workflow run queued at all** — with no explanation
available at the time: the workflow was active, its trigger correct on the
default branch, and GitHub had registered two `release … published` events, yet
no run started on either attempt. *(Resolved: replacing the release-event
trigger with `push: tags:` removed the condition entirely. It has not recurred
across five subsequent tag pushes.)*

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

- **RFC 034 — release workflow reliability.** *Shipped in 0.38.0.*
  Tag-push-triggered release creation removes causes 1–3 structurally — the tag
  carries the workflow that will run, the trigger is `push`, and the ref is a
  tag by construction. All five variants must build before the release is
  created, so a failure leaves no release rather than a partial one. Plus
  `--clobber` for idempotent re-runs, asset-count verification, a
  manifest-must-equal-tag gate, and pre-tag build verification via
  `workflow_dispatch`. **Cause 4 did not recur** — see this theme's Status.
  - Pre-release tag support was accepted, implemented, then withdrawn: internal
    dependencies carry `version = "0"` per RFC 030, and a Cargo requirement
    without a pre-release component never matches a pre-release version, so no
    pre-release could build. Recorded in the RFC rather than deleted.
- **RFC 035 — similarity-dialog cache-error routing.** *Implemented.* Closes
  RFC 033's Part B deferral: cache-read failures in the similar-pairs and focus
  dialogs no longer render as an empty result indistinguishable from "nothing
  found".
- **RFC 038 — native smoke on CI runners** (previously "route B"). *Shipped in
  0.39.1.* Extends automated native smoke to the `windows-latest` and
  `macos-latest` runners the release workflow already uses, covering the two
  highest-value unverified targets without owning hardware. It cannot cover
  rendered UI, which stays desktop-only and human-run.

**Evidence this theme is worth its cost.** Four defects were found during the
0.37.0 cycle — a release-blocking startup hang, a missing required action, a
prohibited affordance, and a false first-run error. **None was found by
automated gates**, which were green throughout the period all four existed.
Three came from rendered smoke; the fourth from the owner running the
application independently.

## Next

### B — Quality debt

**Status.** Agreed 2026-08-03. Follows theme A.

Only the ELOC watch item remains open; everything else in this theme has
shipped.

- **~~Similarity-dialog cache-error tier routing (RFC 035).~~** *Shipped.*
  Closed RFC 033's Part B deferral. Cache-read failures in both similarity
  dialogs now surface as one inline message per dialog open, with partial
  results retained.
- **~~Similarity-dialog absence states
  ([RFC 036](./rfcs/done/036-similarity-dialog-absence-states.md)).~~**
  *Shipped in 0.39.0.* RFC 035 gave failure a voice; this gave the silent
  states one. Both dialogs now distinguish results, failure, nothing-indexed,
  nothing-found, and video-unavailable, from one shared mechanism.
  **Carried forward and now closed:** that text was legible only where it
  landed on neutral background, because `snora`'s dialog overlay drew no card.
  Reported upstream, fixed upstream, and adopted in 0.39.1 via
  [RFC 040](./rfcs/done/040-snora-0.29-upgrade-and-dialog-surface.md) — which
  also closed a live accessibility defect the report surfaced: modals had **no
  modality signal at all** on the high-contrast dark preset.
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

**0.40.0** shipped [RFC 041](./rfcs/done/041-application-data-locations.md):
settings, models and cache move to platform-standard per-user locations, with
everything migrated rather than abandoned. A minor bump rather than a patch
because a user's data relocates, which a patch version would understate. Its
verification failed first on **macOS and Windows for two different reasons** —
an environment variable leaking between parallel tests, and a marker written to
stderr while the workflow grepped stdout — neither a defect in the change, both
defects in checking it, and neither visible on Linux.

**0.39.1** shipped RFCs 038, 039 and 040 — a fix-only release. Its headline is
an accessibility defect: modals had no modality signal at all on the
high-contrast dark preset, the one chosen by users who most need visual
clarity. Its release gate also caught this project's first **vulnerability**
rather than a warning (`webbrowser`, RUSTSEC-2026-0257), closed by a
semver-compatible bump.

**0.39.0** shipped RFCs 036 and 037. It is the first release cut under
draft-then-publish, and the first whose release path had already been exercised
before the tag was pushed.

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
