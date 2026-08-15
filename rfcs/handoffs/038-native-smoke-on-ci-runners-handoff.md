# RFC 038 Handoff — Native smoke on CI runners

Companion to [RFC 038](../proposed/038-native-smoke-on-ci-runners.md), which is
**accepted for implementation** (owner, 2026-08-10) and stays in
`rfcs/proposed/` until the work ships, per
[RFC 000](../done/000-rfc-lifecycle-policy.md).

## 1. Design authority

1. [RFC 038](../proposed/038-native-smoke-on-ci-runners.md) — the governing
   design;
2. [`native-smoke-risk-acceptance`](../notes/native-smoke-risk-acceptance.md) —
   the four residual risks this discharges, and the record that must not be
   rewritten;
3. [RFC 032](../done/032-cross-platform-external-ffmpeg.md) — the trust boundary
   whose guidance the workflow itself must honour.

## 2. The one rule that outranks the rest

**Do not touch `.github/workflows/release-executable.yaml`.**

Not to add a job, not to share a step, not to "reuse" the build matrix. That
file took four tag pushes and three defects to stabilise and now carries RFC
034's blocking gate and RFC 037's publication atomicity. This work introduces
platform- and network-dependent failure surface, and none of it may sit in the
path that publishes.

New file: `.github/workflows/native-smoke.yaml`. `msrv.yaml` is the precedent
for a narrow, separately-scoped workflow.

## 3. Settled design questions — do not re-open

**3.1 Informational, not blocking.** A smoke failure turns the run red and
nothing else. It gates no release and blocks no merge. These jobs depend on
third-party package installs on ephemeral runners; a release channel a Homebrew
outage can block is worse than one that reports honestly. Revisit when the
observed flake rate is a fact rather than a guess.

**3.2 A second test entry point, not a parameterised one.** Variant 2
(discovery) needs its own `#[test] #[ignore]` function alongside the existing
`selected_external_pair_generates_probes_and_extracts_real_video`, so a failure
names which mode broke. Reuse the existing helpers; do not restructure the file.

**3.3 The risk-acceptance note is appended to, never rewritten.** It is a
decision record. Add a dated section in the same shape as its existing
"Re-confirmation — 2026-08-03", naming which rows are discharged, by which run,
on which date. **Do not edit its original rows to say "pass"** — that would
make it read as though the risk was never taken.

**Only discharge rows you actually covered.** The rendered-GUI row stays
`not run` on every platform. Intel macOS and Linux aarch64 stay closed by
decision, untouched.

## 4. The Windows package source

RFC 038 left this open. Settled: **prefer `winget`**, Microsoft's own package
manager, as the closest Windows analogue to Homebrew on macOS — a first-party
source, which is what RFC 032 asks arama's own users to use.

**Verify what is actually on the runner image rather than assuming.** GitHub
publishes the installed tool list per image; check it. If `winget` is
unavailable or unusable non-interactively, Chocolatey is an acceptable fallback.

**Do not download an `ffmpeg.exe` from an arbitrary URL.** A workflow that
fetches an unverified binary would contradict the contract this project asks its
users to honour, in the very test that exists to verify that contract.

## 5. Required implementation

Two variants per platform, on `windows-latest` and `macos-latest`:

**Variant 1 — selected directory.** Install a pair, point
`ARAMA_FFMPEG_SMOKE_DIR` at it, run the existing test.

**Variant 2 — discovery with the pair off `PATH`.** The pair installed but not
on `PATH`, so discovery must find it by the platform fallback. **This is the
point of the RFC** — variant 1 alone leaves the highest named risk, the macOS
fallback, entirely unexercised.

### Variant 2 must prove it took the fallback

The single most important instruction in this handoff:

> **A variant-2 pass that silently used a pair still on `PATH` proves nothing,
> and is worse than not running the test — it converts absent evidence into
> false confidence.**

The run must make the distinction visible. Before the test, log the effective
`PATH` and assert `ffmpeg`/`ffprobe` are *not* resolvable on it; have the test
report which directory discovery returned. A reviewer must be able to see, from
the log alone, that the fallback was the thing that worked.

### Record the version token

Log the `ffmpeg` version string each runner installed. That is the
version-family evidence for the fourth residual risk; without it that row cannot
be discharged.

## 6. Non-change scope

- `release-executable.yaml` — §2.
- Product code. **If the boundary cannot be exercised as shipped, that is a
  finding to report, not a refactor to perform.**
- Rendered GUI verification. CI cannot drive it; it stays human-run.
- Intel macOS, Linux aarch64.
- The risk-acceptance note's existing content.

## 7. If the smoke finds a real bug

Treat it as a finding, report it, and stop. Do not fix a platform defect inside
this task.

This is a live possibility, not a formality: the macOS fallback is the highest
user-visible risk on the accepted list and has never executed anywhere. A red
run on its first execution would be this RFC working, not failing.

## 8. Acceptance criteria

- `native-smoke.yaml` exists; `release-executable.yaml` is byte-identical.
- Both variants run on `windows-latest` and `macos-latest`.
- Variant 2's log demonstrates the pair was **not** on `PATH` and shows which
  directory discovery returned.
- The installed `ffmpeg` version token is recorded per platform.
- The risk-acceptance note gains an appended, dated discharge section covering
  only the rows actually exercised.
- No product-code change.

## 9. Required deliverables

A review request under `.git-exclude/review-request/NNN-<slug>/` opening with an
`## Entry point` section giving the pinned commit, the exact `git show` command,
and plain paths to every file. Include the run IDs, the per-platform version
tokens, and the variant-2 log excerpt proving the fallback was exercised. Report
observed output; a check not run is recorded as not run.
