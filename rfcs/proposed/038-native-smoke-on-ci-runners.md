# RFC 038: Native smoke on CI runners

**Status.** Proposed — awaiting owner decision. Not accepted.
**Tracks.** Discharge the residual risks the owner accepted in
[`native-smoke-risk-acceptance`](../notes/native-smoke-risk-acceptance.md) by
exercising arama's external-FFmpeg boundary on `windows-latest` and
`macos-latest` — the runners the release workflow already pays for. Theme A's
last unwritten item.
**Touches.** A new workflow file, and possibly
`crates/engine/sidecar/tests/external_ffmpeg_smoke.rs`. **No product code, and
no change to `release-executable.yaml`.**

## Summary

Arama's FFmpeg discovery boundary has never executed on Windows or macOS. Every
automated test of it runs on Linux and covers *logic* — PATH parsing,
normalization rules, timeout arithmetic — not platform behaviour. The two risks
that matter are behavioural, and the owner accepted them explicitly for the
0.37.0 checkpoint on the understanding that a follow-up RFC would discharge
them.

This is that follow-up. A dedicated workflow runs the existing
`#[ignore]`-gated smoke test on `windows-latest` and `macos-latest`, plus a
`PATH`-stripped variant that forces the macOS Homebrew-prefix fallback to be the
thing under test.

## Why now

The risk-acceptance note says the residual risk is **temporary, not permanent**,
and names this work as what makes it so. It also states the acceptance covers
"this release checkpoint only." Two releases have shipped since.

The specific accepted risks, from that note:

| Platform | Accepted risk | Worst case |
|---|---|---|
| Windows | job-object process-tree reaping via `command-group`, unverified on the platform | a leaked `ffmpeg` after a probe timeout |
| Windows | filesystem identity and canonicalization use different APIs than the Linux logic exercises | a valid installed pair rejected — user sees "ffmpeg not found" while having it |
| macOS | the Homebrew-prefix fallback exists solely for the non-inherited-`PATH` case and can only be verified on the platform | **every** macOS user launching without an inherited `PATH` sees video features unavailable despite a working install |
| Both | version-token parsing exercised against one build family (`n8.1.2`) only | an unrecognised token from a Homebrew or Windows distribution build |

The macOS row is the note's own "highest user-visible risk on the list."

**One correction the note already records:** arama ships no `.app` bundle, so
the Finder-launch scenario that originally motivated the fallback is not
arama's distribution format. The fallback still has value — a stripped or
minimal `PATH` reaches the same code — but this RFC should test *that*
condition rather than a Finder launch it cannot reproduce.

## Current coverage

`crates/engine/sidecar/tests/external_ffmpeg_smoke.rs` holds a single test,
`selected_external_pair_generates_probes_and_extracts_real_video`, gated:

```rust
#[ignore = "owner-run native smoke; set ARAMA_FFMPEG_SMOKE_DIR to a trusted pair"]
```

It validates a selected directory, generates a real fixture, reads duration via
`ffprobe`, and extracts a frame using only the returned toolchain authority. It
has been run once, on Linux, against `/usr/bin` with `n8.1.2`.

**The mechanism already exists.** This RFC is mostly about where it runs.

## Goals

- Execute the existing smoke test on `windows-latest` and `macos-latest`.
- Force the Homebrew-prefix / minimal-`PATH` path to be exercised rather than
  bypassed by a runner that happens to have `ffmpeg` on `PATH`.
- Confirm version-token parsing against at least one non-Linux build family per
  platform.
- Leave a durable record so the risk-acceptance note's rows can be superseded by
  evidence rather than by assertion.

## Non-goals

- **Any change to `release-executable.yaml`.** See Design.
- Rendered GUI verification. CI cannot drive it; it stays desktop-only and
  human-run, exactly as the note says.
- Intel macOS and Linux aarch64. Both are **closed by decision**, not pending,
  and this RFC does not reopen them.
- Product code changes to make testing easier. If the boundary cannot be
  exercised as shipped, that is a finding, not a refactor prompt.

## Design

### A separate workflow, not the release workflow

**This is the load-bearing decision.** `release-executable.yaml` took four tag
pushes and three defects to stabilise across the 0.38.0 cycle, and now carries
RFC 034's blocking gate and RFC 037's atomicity. It is the mechanism by which
all work reaches users.

Adding smoke jobs to it would put new, platform-dependent, externally-dependent
failure surface **inside the path that publishes**. A flaky Homebrew install
would then be able to block a release.

New file — `native-smoke.yaml` — triggered on `workflow_dispatch` and on push
to `main`. `msrv.yaml` is the precedent for a narrow, separately-scoped job.

### Getting a real ffmpeg pair on each runner

The test needs a trusted pair at a known directory. Per platform:

- **macOS**: Homebrew is present on `macos-latest`; `brew install ffmpeg`
  yields a pair under the Homebrew prefix — which is exactly the directory the
  fallback is written to find.
- **Windows**: the runner image's available package managers should be used
  rather than downloading a binary from an arbitrary URL. **Whatever is chosen
  must be a trusted source**, consistent with RFC 032's user-facing guidance;
  a workflow that fetches an unverified `ffmpeg.exe` would contradict the
  contract this project asks its users to honour.

Both are network installs on ephemeral runners, so they are the least stable
part of this design — see Risks.

### Two variants per platform

1. **Selected directory** — `ARAMA_FFMPEG_SMOKE_DIR` pointed at the installed
   pair. Exercises validation, probe, and extraction.
2. **Discovery with a stripped `PATH`** — the pair installed but *not* on
   `PATH`. This is the variant that makes the macOS fallback the thing under
   test rather than an unused branch, and it is the closest reproducible
   analogue of the non-inherited-`PATH` condition.

Variant 2 is the point of the RFC. Variant 1 alone would leave the highest
named risk untouched.

## Design questions this RFC must settle

### 1. Does a smoke failure block anything?

Options: informational (red run, no gate), or a release precondition.

**Recommendation: informational, initially.** These jobs depend on third-party
package installs on ephemeral runners, and a channel that can be blocked by a
Homebrew outage is worse than one that reports honestly. Revisit once the
observed flake rate is known rather than guessing it now.

### 2. Does the existing test need to change?

It is `#[ignore]`d and env-driven, which is already CI-shaped. Variant 2 may
need either a second test or a second env var to assert that discovery — not
selection — found the pair.

**Prefer a second entry point over parameterising the existing one**, so a
failure names which mode broke.

### 3. What supersedes the risk-acceptance note?

That note is a **decision record** and must not be rewritten to claim risks were
never taken. Recommendation: leave it intact and append a dated section
recording which rows are discharged, by which run, on which date — the same
shape as its existing "Re-confirmation — 2026-08-03" section.

## Testing and verification

The workflow is its own verification: it either runs the smoke test on both
platforms or it does not.

- First green run on each platform, with the observed `ffmpeg` version token
  recorded — that is the version-family evidence.
- Variant 2 must be shown to have actually taken the fallback path, not merely
  passed. A pass that silently used a pair still on `PATH` proves nothing; the
  run log must make the distinction visible.
- Run the workflow **before** proposing to supersede any risk row.

## Acceptance criteria

- The smoke test executes on `windows-latest` and `macos-latest`.
- The stripped-`PATH` variant demonstrably exercises discovery, not selection.
- `release-executable.yaml` is unchanged.
- Observed version tokens are recorded per platform.
- The risk-acceptance note gains an appended, dated discharge section naming the
  runs — and only for rows actually covered.
- Rendered-GUI rows remain `not run`; nothing here implies otherwise.

## Risks

- **Third-party install flake.** The most likely failure mode is `brew` or the
  Windows package source, not arama. Design question 1 exists for this.
- **A pass that proves less than it appears to.** If the runner already has
  `ffmpeg` on `PATH`, variant 2 can pass without touching the fallback. This is
  the one way this RFC could deliver false confidence, and it is worse than not
  running it — the acceptance criteria call it out for that reason.
- **Scope creep toward "CI runs everything."** This RFC discharges four named
  risks. It is not a mandate to move the smoke checklist onto CI.
- **A finding is a real possibility.** The macOS fallback is the highest
  user-visible risk on the list and has never executed. This RFC may produce a
  bug rather than a green tick — which is the point.

## Open questions

- Which Windows package source meets "trusted" in the same sense RFC 032 asks of
  users?
- Should the workflow run on every push to `main`, or on a schedule? Per-push is
  simpler and the cost is small; a schedule reduces noise if flake proves
  common.
