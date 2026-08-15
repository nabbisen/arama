# Native smoke coverage: owner risk acceptance (external FFmpeg, RFC 032)

**Status.** Decision record
**Date.** 2026-08-01
**Decided by.** Project owner, on the architect's cost/benefit recommendation.
**Applies to.** The RFC 032 external-FFmpeg release checkpoint (ROADMAP
milestones 3 and 5).

## Re-confirmation — 2026-08-03

The owner re-confirmed this acceptance **knowing** what the Linux smoke run
subsequently found. The original decision below is unchanged; this section
records what changed around it, so nobody reads the acceptance as having been
made in ignorance.

**Three findings surfaced after this note was written**, all from Task 004's
rendered smoke and its follow-ups:

1. a release-blocking hang — a valid persisted Selected-directory preference
   never resolved on startup or re-check (review 067 Finding 1, fixed at 069);
2. a missing required Setup action — Select directory (Finding 2, fixed at 070);
3. a prohibited affordance — a progress bar and MB label on the FFmpeg Setup row
   (Finding 3, fixed at 071).

**None of them implicates the risks accepted below.** All three were
platform-independent, all were found *by* the Linux testing this note authorised,
and none touches Windows job-object reaping, filesystem identity semantics, or
macOS launch-context `PATH` behaviour. What they revised was confidence in the
implementation generally — not the specific residual risk list.

**One risk is narrower than stated below.** The macOS entry cites RFC 031's
rationale that "a `.app` launched from Finder may not inherit the interactive
shell's `PATH`." Arama ships **no `.app` bundle** — the release workflow produces
a bare binary inside a wrapping directory in a `.zip`. A user launching from a
terminal inherits `PATH` normally. The Homebrew-prefix fallback retains value,
but the scenario that motivated it is not arama's distribution format.

**The residual risk is temporary, not permanent.** Route B (native smoke on the
`windows-latest` and `macos-latest` runners already used by the release
workflow) discharges essentially all of it: PATH discovery, pair validation,
timeout and descendant reaping, real-media probe and extraction, and — by
stripping `PATH` on the runner — the macOS prefix fallback itself. What CI
cannot drive is a rendered GUI, which a human on either platform could do at any
time. Intel macOS and Linux aarch64 remain closed by decision rather than
pending.

Condition 1 below still governs: this acceptance covers **this release
checkpoint only**.

## Discharge — 2026-08-15

RFC 038 landed and ran for real on `windows-latest` and `macos-latest`
(`.github/workflows/native-smoke.yaml`), discharging most of what the
"Follow-up" section below anticipated. This section records which rows
that changes, and does not edit the original rows above to say "pass" —
they remain the record of what was knowingly not verified at the time this
acceptance was made.

**Runs:** [31863143720](https://github.com/nabbisen/arama/actions/runs/31863143720)
(both `macos` and `windows` jobs green; three earlier attempts on the same
day found and fixed bugs in the workflow script itself, not in arama — see
the RFC 038 review package for the full sequence).

**Apple Silicon macOS — discharged.** Variant 1 (selected directory)
passed: real fixture generated, `ffprobe` duration read, frame extracted,
using a Homebrew-installed pair (`ffmpeg`/`ffprobe` 8.1.2). Variant 2
(discovery with `PATH` stripped) proved the native-prefix fallback is what
actually resolved the pair, not an accident of a runner that happened to
have `ffmpeg` on `PATH`:

```
NATIVE_SMOKE_DISCOVERY_OUTCOME=Ready
NATIVE_SMOKE_DISCOVERY_SOURCE=NativePrefix
NATIVE_SMOKE_DISCOVERY_PATH=/opt/homebrew/bin/ffmpeg
```

This was the highest-named risk on the list and the RFC's actual point.
**Discharged by evidence**, not assertion.

**Windows x86_64 filesystem identity / pair validation — discharged for the
Selected-directory path.** Variant 1 passed against a real Chocolatey
install (`ffmpeg`/`ffprobe` 9.0.1, `gyan.dev` build), exercising Windows
filesystem-identity and canonicalization logic against a real installed
pair rather than the Linux-only logic tests. A valid pair was recognized,
not rejected.

**Windows x86_64 process-tree reaping — still not discharged.** Neither
variant deliberately produced a probe timeout; Variant 1 exercised normal
spawn/wait (fixture generation, probe, extraction all completed quickly).
Basic process handling on Windows is now evidenced; the specific
timeout-then-reap path job objects are meant to handle is not. This risk
remains open.

**Version-token parsing — discharged against two new build families.**
The original acceptance had one data point (Linux, `n8.1.2`). This run
adds two more, both successfully parsed (Variant 1 passing requires
`parse_version_token` to succeed): macOS Homebrew's `8.1.2` (no `n`
prefix — a different format than Linux's own build) and Windows's
`9.0.1-essentials_build-www.gyan.dev` (a third, more elaborate format).
Three real formats now confirmed parseable.

**A new finding, not one of the four originally accepted risks.** Windows
Variant 2's auto-discovery outcome was not the expected `Missing` (RFC 032
gives Windows no automatic off-`PATH` fallback, so `Missing` was predicted
as the correct behaviour) — it was `SearchLimitReached(CandidateCount)`.
The GitHub-hosted `windows-latest` runner's real `PATH` has **71 raw
entries**, and `FfmpegLocatorPolicy::default().max_raw_path_entries` is
**64** (`crates/engine/sidecar/src/media/video/video_engine/discovery/policy.rs`).
Discovery truncated the raw list before reaching the point of concluding
`Missing`.

This is not a bug this task fixes — per RFC 038 handoff §7 and non-change
scope, a platform finding is reported, not repaired inside this task. It
is recorded here because it is new information the original four-risk list
did not anticipate: **a real, unremarkable enterprise/developer Windows
machine can have a `PATH` long enough that arama's own search-bound policy
gives up before exhausting it** — independent of whether a valid pair
would have been found further down the list. The 71-entry `PATH` observed
here is not synthetic or adversarial; it is what a heavily-provisioned CI
runner's default environment looks like, and is plausible for real
developer machines with many SDKs installed. Whether `max_raw_path_entries`
is still calibrated correctly, and to what, is a design question for a
future task — not this one.

**Update, same day — Windows x86_64 process-tree reaping — now discharged
(Task 021).** The paragraph above records what was true when this section
was first written; it is left as-is rather than edited, per this note's own
rule that it is appended to, never rewritten.

Task 021 drove real Selected-directory discovery (the public path;
`run_bounded_probe_with_cancellation` was deliberately not widened past
`pub(super)` to make this easier — that would have been a design decision,
not a test) at a hanging `ffmpeg`/`ffprobe` stub compiled directly with
`rustc` on both runners (not a workspace member — a text-script stub cannot
be a native Windows executable). The stub spawns a grandchild of itself
before hanging, so the test proves something about the *tree*, not the
direct child: a `kill()` that only reached the process arama spawned itself
was never in doubt and would have proven nothing.

Both platforms, run
[31880249221](https://github.com/nabbisen/arama/actions/runs/31880249221):

```
NATIVE_SMOKE_REAPING_GRANDCHILD_PID=2328               (macOS)
NATIVE_SMOKE_REAPING_GRANDCHILD_ALIVE_BEFORE_TIMEOUT=true
NATIVE_SMOKE_REAPING_TERMINAL_OUTCOME=ProbeTimedOut
NATIVE_SMOKE_REAPING_GRANDCHILD_ALIVE_AFTER_TIMEOUT=false

NATIVE_SMOKE_REAPING_GRANDCHILD_PID=8016               (Windows)
NATIVE_SMOKE_REAPING_GRANDCHILD_ALIVE_BEFORE_TIMEOUT=true
NATIVE_SMOKE_REAPING_TERMINAL_OUTCOME=ProbeTimedOut
NATIVE_SMOKE_REAPING_GRANDCHILD_ALIVE_AFTER_TIMEOUT=false
```

The grandchild's identity was confirmed alive before the timeout — so the
tree under test genuinely existed — and confirmed absent after, on both
platforms. **`command-group`'s Windows job-object path reaps the full
process tree on a probe timeout, matching its already-trusted Unix
process-group path.** The worst case this risk described — a leaked
`ffmpeg.exe` after a probe timeout — is not what happens.

This closes the last of the four risks accepted for the 0.37.0 checkpoint.

**Not discharged, unchanged from the original acceptance:** rendered setup
and Settings UI states (CI cannot drive a GUI on any platform, exactly as
this note always said); Intel macOS and Linux aarch64 (closed by decision,
not reopened).

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
