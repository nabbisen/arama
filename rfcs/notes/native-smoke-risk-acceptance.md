# Native smoke coverage: owner risk acceptance (external FFmpeg, RFC 032)

**Status.** Decision record
**Date.** 2026-08-01
**Decided by.** Project owner, on the architect's cost/benefit recommendation.
**Applies to.** The RFC 032 external-FFmpeg release checkpoint (ROADMAP
milestones 3 and 5).

## Why this record exists

ROADMAP milestone 5 requires that every unavailable native target row carry an
**explicit owner risk acceptance rather than an inferred pass**. This file is
that acceptance. It lives in `rfcs/notes/` rather than only in the roadmap
because roadmaps get rewritten and this must remain findable afterwards.

Nothing here should be read as evidence that an unexecuted target passed.

## The decision

For the current release checkpoint: **exercise Linux x86_64 only; record every
other target as `not run` with the risk accepted below.** Extending automated
native smoke to CI runners is deferred to a follow-up RFC, to land before the
*next* release rather than blocking this one.

## Coverage and rationale

| Target | Result | Rationale |
|---|---|---|
| Linux x86_64 | **pass** (real-media, 2026-08-01) | Maintainer's own platform; free to exercise |
| Windows x86_64 | **not run** — risk accepted | Highest unique coverage, but needs hardware or a CI job that does not exist yet |
| Apple Silicon macOS | **not run** — risk accepted | High unique coverage; same constraint |
| Intel macOS | **not run** — permanent | Differs from Apple Silicon by one prefix constant (`/usr/local/bin`). Near-zero incremental value; not treated as pending |
| Linux aarch64 | **not run** — permanent | Same OS and process/path semantics as x86_64. CPU architecture is irrelevant to the FFmpeg discovery boundary. Not treated as pending |

The last two rows are deliberately closed, not deferred. Leaving them open
would imply work that nobody intends to do.

## What is specifically accepted

Automated Linux-host tests cover PATH parsing, deduplication, Windows
case/separator/UNC normalization **as logic**, coordinator cancellation,
preference transactions, outcome classification, legacy exclusion, and bounded
probe timeouts. They cannot cover platform execution behaviour. The accepted
residual risks are therefore:

**Windows x86_64**

- process-tree termination and reaping uses job objects rather than Unix
  process groups; `command-group` abstracts this and the abstraction is
  unverified on the platform. Worst case: a leaked `ffmpeg` process after a
  probe timeout.
- filesystem identity and canonicalization use different APIs than the logic
  exercised on Linux. Worst case: a valid installed pair is rejected, or probed
  twice, and the user sees "ffmpeg not found" while having it installed.

**Apple Silicon macOS**

- a Finder-launched application does not inherit the interactive shell's
  `PATH`. The native Homebrew-prefix fallback exists solely for this case and
  can only be verified on the platform. Worst case: every macOS user launching
  from Finder sees video features unavailable despite a working Homebrew
  install. This is the highest user-visible risk on the list, mitigated only by
  the fact that the path was the most heavily reviewed part of RFC 031.

**Both**

- version-token parsing is exercised against one real build family only. The
  Linux run matched `n8.1.2`, so a build-prefixed token is known to parse;
  Homebrew and Windows distribution formats are not confirmed.
- rendered setup and Settings UI states are unverified anywhere — no automated
  UI coverage exists on any platform.

## Evidence supporting the executed row

- `ARAMA_FFMPEG_SMOKE_DIR=/usr/bin cargo test -p arama-sidecar --test
  external_ffmpeg_smoke -- --ignored --exact
  selected_external_pair_generates_probes_and_extracts_real_video` — **pass**,
  2026-08-01. Selected-directory validation, then real fixture generation,
  `ffprobe` duration read, and frame extraction using only the returned
  toolchain authority. Local pair: `/usr/bin/ffmpeg` and `/usr/bin/ffprobe`,
  both `n8.1.2`.
- `scripts/check-external-ffmpeg-contract.sh` — **pass**. Production source,
  sidecar dependency stack, and all ten `cargo package --list` outputs contain
  no FFmpeg acquisition identifier or payload.
- Archive and built-executable inspection (`--archive`, `--binary`) —
  **not run**; those artifacts do not exist until release packaging. Deferred
  to the release step.

## Conditions on this acceptance

1. It covers **this release checkpoint only**. A later release does not inherit
   it; the follow-up CI work is expected to replace it.
2. It does not authorize a release. Release approval remains a separate owner
   decision.
3. If any Windows or macOS defect matching the risks above is reported, this
   acceptance is the record of what was knowingly not verified — not a defence
   that it was tested.
4. Should a Windows machine or Mac become available before release packaging,
   running the smoke there is cheap and supersedes the corresponding row.

## Follow-up

Extend native smoke to the `windows-latest` and `macos-latest` runners the
release workflow already uses, so the two highest-value targets are covered
without owning hardware. This requires its own RFC: RFC 033 Part F explicitly
fenced CI expansion as a separate theme, and the work is not merely a job
addition — the runners must install a trusted pair and the ignored smoke must
be parameterised for them. It cannot cover Finder-launch `PATH` inheritance or
rendered UI, which remain desktop-only.
