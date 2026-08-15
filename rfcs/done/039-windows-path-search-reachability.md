# RFC 039: Windows `PATH` search reachability

**Status.** Implemented (0.39.1). Accepted by the project owner 2026-08-15.

*Phase 0 measured before the value was chosen*, as the handoff required:
`windows-latest` presents **78 raw entries / 66 unique candidates**;
filesystem collection costs ~51–85µs per entry on Windows against ~3.9µs on
Linux; a full all-miss attempt takes 8–12ms against a 6-second budget. The
bounds are **256 / 128** on Windows only — roughly 3× and 2× headroom over
what was observed — and `attempt_timeout` is unchanged, now on measurement
rather than the plausibility argument this RFC raised.

*§3.2 was proven, not assumed:* an instrumented run with the raw cap at 512 and
`max_path_candidates` left at 32 still returned
`SearchLimitReached(CandidateCount)`, because 66 real candidates already exceed
32. Both bounds moved.

*This RFC's own §5 prediction was wrong.* Raising the bounds surfaced
`FilesystemUnavailable(MetadataOrIdentity)` rather than `Missing` — a
pre-existing defect that truncation had been masking, in which a single
dangling `PATH` entry made the "install ffmpeg" message unreachable. Fixed
under Task 019; the prediction became true only afterwards.
**Tracks.** Amend [RFC 032](../done/032-cross-platform-external-ffmpeg.md)'s
discovery bounds. `max_raw_path_entries: 64` is a **reachability** limit, not a
performance limit, and on Windows it is the only thing standing between a user
and their installed ffmpeg in Auto mode.
**Touches.** `crates/engine/sidecar/src/media/video/video_engine/discovery/policy.rs`,
its tests, and RFC 032's bounds table. No UI, no workflow.

## Summary

Auto discovery caps the `PATH` scan at 64 raw entries. The cap is applied with
`.take(64)` on the iterator **before any entry is inspected**, so a valid ffmpeg
directory at raw position 65 is never canonicalized, never checked, and never
becomes a candidate — because of its position, not because anything about it
failed.

On macOS this is harmless. On Windows it is not, and the difference is
structural rather than statistical.

## Why now

RFC 038's native smoke ran Auto discovery on `windows-latest` with the pair off
`PATH` and got `SearchLimitReached(CandidateCount)` rather than the predicted
`Missing`. The runner presents **71** raw `PATH` entries against a cap of 64.

Task 018 then traced the mechanism. Its conclusion — that this is a
reachability bound rather than a performance bound — is the finding, and it
changes what the fix has to be.

## The asymmetry that makes this Windows-specific

`normalize_auto_candidates` (`path_policy.rs:85-137`) does two things in order:

1. iterate `PATH`, bounded by `.take(policy.max_raw_path_entries)`;
2. **after** that loop, append the native-prefix candidate, exempt from the cap.

And `native_prefix()` (`worker.rs:294`) returns:

| Platform | Value |
|---|---|
| macOS aarch64 | `/opt/homebrew/bin` |
| macOS x86_64 | `/usr/local/bin` |
| **Windows** | **`None`** |
| **Linux** | **`None`** |

So:

- **macOS** has a cap-exempt reserved slot. Even a 500-entry `PATH` still finds
  a Homebrew install. RFC 038 demonstrated exactly this — variant 2 resolved via
  `NativePrefix` with `PATH` stripped entirely.
- **Windows** has no reserved slot and no fallback. The capped scan is the
  **only** Auto-mode route.
- **Linux** also has no reserved slot, but short `PATH`s are the norm there and
  no evidence suggests the cap is reached in practice.

**This argument does not depend on how common 71-entry `PATH`s are.** Windows is
the one platform where the bound is load-bearing for reachability and the one
platform with nothing behind it. The `PATH`-length evidence — one CI measurement
plus character-length literature — corroborates but is not required.

## What the bound is actually for

RFC 032 states the purpose, not the arithmetic. The bounds exist to keep an Auto
attempt inside a fixed 6-second wall-clock budget and to cap filesystem and
subprocess work — a UX concern (never leave "Checking" hanging) and a defensive
one (RFC 032's security section cites bounded probes as mitigation against an
attacker-influenced `PATH` forcing unbounded subprocess execution).

**Nothing anywhere derives 64 from an expected `PATH` length.** There is no line
saying "`PATH` rarely exceeds N entries." The numbers read as a round defensive
ceiling. That is headroom: raising the raw cap does not abandon the bound's
purpose, provided total wall-clock stays bounded.

## Goals

- A Windows user with ffmpeg on a normal-length `PATH` is reachable by Auto
  discovery.
- The wall-clock and subprocess bounds RFC 032 established still hold.
- Whatever changes is recorded where RFC 032's reasoning lives, not only in
  `policy.rs`.

## Non-goals

- Removing the bound. It is defensive and stays.
- Changing macOS or Linux behaviour. No evidence supports moving them, and the
  timeout interaction below is Windows-specific too.
- Windows process-tree reaping on a probe timeout — still open from RFC 038,
  separate, not folded in.
- Making Windows Auto mode find pairs that are genuinely off `PATH`. RFC 032
  gives Windows no off-`PATH` fallback by design; Selected-directory mode is
  that path and it works.

## Design questions this RFC must settle

### 1. What should the Windows bound be?

Raising it to a value comfortably above observed real `PATH`s — 71 on
`windows-latest` — with headroom, rather than a precisely fitted number. The
bound's job is to be a ceiling, not an estimate.

**A single raised default versus a per-platform default.** Per-platform is
recommended: the timeout interaction in question 2 is Windows-specific, so a
blanket raise applies a risk to platforms where it is not evidenced. This would
introduce the **first platform-conditional policy value** — today
`FfmpegLocatorPolicy::default()` is a single unconditional impl — which is a
design change and part of why this is an RFC.

### 2. Does `attempt_timeout` move with it?

**This is the question that makes the fix non-trivial.**

`attempt_timeout` (6s) is a single **shared, whole-attempt** budget: `started`
is set once (`worker.rs:37`) and every checkpoint compares against it. Raising
the raw cap therefore spends more of the same fixed budget. If Windows
per-candidate filesystem and process operations are slower — plausible, given
job-object process spawn and different filesystem APIs, and **unmeasured
anywhere in this codebase** — a raised cap could convert "gives up early,
correctly reported" into "times out mid-scan on a legitimately long `PATH`".

That is a different degraded case, not obviously a better one.

**Recommendation: measure before choosing.** The native-smoke workflow from
RFC 038 already runs on `windows-latest` and can time a full Auto attempt over a
long `PATH`. Guessing here would repeat the original sin — a round number with
no derivation.

### 3. Also: `max_path_candidates: 32`?

A second, independent cap on the deduplicated candidate list (`path_policy.rs:108`),
which sets `candidate_truncated` and produces the same
`SearchLimitReached(CandidateCount)` outcome. Forty unique valid directories
inside the first 64 raw entries yields 32 kept and 8 dropped.

Whether it needs to move too depends on question 1's answer. **Do not raise the
raw cap while leaving this one such that the second cap simply becomes the new
ceiling** — that would produce a change with no observable effect.

## Testing and verification

- **A test at the real default boundary.** Task 018 found that nothing in the
  suite exercises this: the raw cap is tested only via a synthetic
  `max_raw_path_entries: 1` override, and the 32-candidate cap only with well
  under 64 raw entries. A regression test must construct a `PATH` at the real
  scale and assert an entry beyond the old cap is now reachable.
- **Timing evidence from `windows-latest`** for question 2, before the value is
  fixed.
- The existing native-smoke variant 2 on Windows should change outcome once the
  bound is raised past the runner's 71 entries — a live end-to-end check that
  costs nothing extra.

## Acceptance criteria

- A valid directory beyond the old cap is reachable by Auto discovery on
  Windows, asserted by a test at the real default scale.
- Total attempt wall-clock still bounded; no "Checking" state can outlive the
  attempt budget.
- macOS and Linux behaviour unchanged.
- RFC 032's bounds table amended to match, with the platform difference and its
  reason stated there.
- `max_path_candidates` addressed rather than silently left as the new ceiling.

## Risks

- **Trading a reachability failure for a timeout failure.** Question 2 exists
  for this; measuring first is the mitigation.
- **A security bound raised without thought.** RFC 032 cites this cap as
  mitigation against an adversarial `PATH`. The raise should be a ceiling with
  headroom, not "large enough that it never triggers" — a bound that never
  triggers is not a bound.
- **Platform-conditional policy as precedent.** Once one value differs by OS,
  others will be proposed. Worth stating in RFC 032's amendment that this one
  differs because the *fallback structure* differs, not as a general licence.

## Open questions

- Should Linux get the same treatment for symmetry, or is the absence of
  evidence there a good enough reason to leave it? Recommendation: leave it, and
  say why, so the asymmetry is deliberate rather than accidental.
