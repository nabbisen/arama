# RFC 039 Handoff — Windows `PATH` search reachability

Companion to [RFC 039](../done/039-windows-path-search-reachability.md),
shipped in **0.39.1** and moved to `rfcs/done/` with that cut, per
[RFC 000](../done/000-rfc-lifecycle-policy.md).

**Do [RFC 040](../done/040-snora-0.29-upgrade-and-dialog-surface.md)
first.** It carries a defect already shipped to users; this one does not.

## 1. Design authority

1. [RFC 039](../done/039-windows-path-search-reachability.md);
2. [RFC 032](../done/032-cross-platform-external-ffmpeg.md) — the bounds table
   this amends and the security reasoning behind it;
3. `.git-exclude/review-request/095-…/README.md` — your own Task 018 analysis,
   which is the evidence base and does not need redoing.

## 2. Phase 0 — measure before choosing the value. Blocking.

**Do not pick a number and then justify it.** That is how 64 arrived: RFC 032
records the bound's *purpose* but no derivation, and the value reads as a round
ceiling. Repeating that would be the same mistake with a bigger constant.

`attempt_timeout` is a **single shared 6-second budget** for the whole attempt —
`started` is set once (`worker.rs:37`) and every checkpoint compares against it.
Raising the raw cap spends more of that same fixed budget. If Windows
per-candidate filesystem and process work is slower than Linux/macOS — plausible
and **unmeasured anywhere in this codebase** — a raised cap converts "gives up
early, correctly reported" into "times out mid-scan on a legitimately long
`PATH`". That is a different degraded case, not a better one.

**What to measure**, on `windows-latest` via the RFC 038 native-smoke workflow
which already runs there:

- wall-clock for a full Auto attempt over the runner's real 71-entry `PATH`;
- per-candidate cost, so the budget headroom at a raised cap can be projected
  rather than hoped for.

**Report the measurement before implementing.** If it shows 6 seconds cannot
comfortably cover the raised cap on Windows, that is a finding that changes this
RFC — stop and report rather than raising `attempt_timeout` on your own
initiative. Moving a timeout is a second design decision, not a consequence of
the first.

## 3. Settled design questions

**3.1 Per-platform, Windows only.** Not a raised shared default. The timeout
interaction above is Windows-specific, so a blanket raise applies a risk to
platforms where none is evidenced. macOS has a cap-exempt native-prefix reserved
slot and does not need it; Linux `PATH`s are short and no evidence suggests the
cap is reached.

This introduces the **first platform-conditional policy value** —
`FfmpegLocatorPolicy::default()` is one unconditional impl today. That is the
design change, and it is why this is an RFC rather than a constant edit.

**3.2 `max_path_candidates: 32` must be addressed, not ignored.** It is a
second, independent cap (`path_policy.rs:108`) producing the same
`SearchLimitReached(CandidateCount)` outcome. **Raising the raw cap while
leaving this one such that it simply becomes the new ceiling would be a change
with no observable effect** — the exact failure this task exists to avoid.

Decide it deliberately: either it moves too, with reasoning, or you show that
32 unique valid directories is not a realistic constraint even on a long `PATH`.
Say which.

**3.3 Leave Linux alone, and say why.** Asymmetry is fine when deliberate.
Record in the RFC 032 amendment that Windows differs because its *fallback
structure* differs — no reserved slot, no off-`PATH` route in Auto mode — not as
a general licence for per-OS tuning.

## 4. Required implementation

Once Phase 0 reports:

- a platform-conditional default for `max_raw_path_entries`, Windows only;
- whatever `max_path_candidates` decision §3.2 produced;
- **RFC 032's bounds table amended** to match, with the platform difference and
  its reason stated there. The table is normative; leaving it asserting 64 while
  the code does otherwise recreates exactly the doc-versus-behaviour defect this
  project reported to snora.

## 5. Testing

**A test at the real default scale.** Task 018 found the gap: the raw cap is
exercised only via a synthetic `max_raw_path_entries: 1` override, and the
32-candidate cap only with well under 64 raw entries. **Nothing in the suite
touches the real default at real scale.**

Construct a `PATH` at genuine scale and assert a valid directory beyond the old
cap is now reachable. That test is the regression guard and it must fail against
the current constant.

**Free end-to-end check:** RFC 038's Windows variant 2 currently returns
`SearchLimitReached(CandidateCount)` against the runner's 71 entries. Once the
bound is raised past that, its outcome should change. Report what it becomes —
`Missing` is the expected and correct result there, since RFC 032 gives Windows
no off-`PATH` fallback by design.

## 6. Non-change scope

- macOS and Linux discovery behaviour.
- Removing or weakening the bound. It is defensive and stays a ceiling with
  headroom — **a bound that never triggers is not a bound.**
- `attempt_timeout`, without a separate decision. See §2.
- Windows process-tree reaping on a probe timeout — still open from RFC 038,
  separate, not folded in.
- Making Windows Auto mode find genuinely off-`PATH` pairs. Selected-directory
  mode is that path and it works.

## 7. Acceptance criteria

- Phase 0's measurement reported before the value was chosen.
- A valid directory beyond the old cap is reachable on Windows, asserted by a
  test at the real default scale that fails against the current constant.
- `max_path_candidates` decided deliberately, with reasoning recorded.
- macOS and Linux unchanged.
- RFC 032's bounds table amended.
- Attempt wall-clock still bounded; no "Checking" state outlives the budget.

## 8. Required deliverables

A review request under `.git-exclude/review-request/NNN-<slug>/` opening with an
`## Entry point` section giving the pinned commit, the exact `git show` command,
and plain paths to every file. Include Phase 0's timing figures, the new test,
and the RFC 038 variant-2 outcome change. Report observed output; a check not
run is recorded as not run.
