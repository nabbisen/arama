> **ACCEPTED 2026-11-15 — review 171** (`.git-exclude/reviewed/171-t076-cache-tests-macos-windows-review.md`).
> Windows grandchild reaping proven on the runner.

> **AUTHORIZED 2026-11-15 — owner:** *"T076 is accepted."* Written 2026-11-15 by
> the architect, after accepting T061 (review 170).
>
> **Independent of every open task.**

# T076 Handoff — run `arama-cache`'s tests on macOS and Windows, and prove Windows reaps a grandchild

## 1. Purpose

Make CI actually test the code T061 added for macOS and Windows. Today it only
compiles it there.

## 2. Background

- T061 (RFC 047) kills a thumbnail ffmpeg **as a process group**
  (`command-group`'s `group_spawn` / `GroupChild::kill`) in
  `crates/cache/src/core/thumbnail.rs`. A group, and not just the direct child,
  because an orphaned grandchild holding the stderr pipe would keep the drain
  thread's `join` waiting. On Windows, `command-group` does this through a job
  object; on Unix, through a process group. The two are different code.
- **Its tests run on Linux only.** `cargo test -p arama-cache` happens only in
  `msrv.yaml`'s `cargo test --workspace` (ubuntu). `native-smoke.yaml`'s macOS
  and Windows jobs test `arama-sidecar`, `arama-env` and `arama --lib` — never
  `arama-cache`. So:
  - the three `#[cfg(windows)]` tests in that file
    (`a_process_that_never_exits_is_killed_within_the_timeout`, and the two exit
    cases) **have never run anywhere**;
  - `killing_the_group_also_reaps_a_grandchild_it_spawned` is `#[cfg(unix)]`. Its
    macOS fix (review 162: file length instead of `date +%s%N`) has never run on
    macOS;
  - **no test proves Windows reaps a grandchild at all.** The existing Windows
    timeout test asserts the call returns quickly, not that `ping` (the child of
    `cmd`) died.
- Found in review 170 (`.git-exclude/reviewed/170-t061-second-rework-review.md`).

## 3. Applicable RFC and requirements

[RFC 047](../done/047-indexing-cancellation.md) — Stop ends the work,
including the external processes it started. The owner's test rules: check each
property at the cheapest level that proves it — here, a test on the platform
whose code it is.

## 4. Change scope

- `.github/workflows/native-smoke.yaml` — one `cargo test -p arama-cache
  --locked` step in the macOS job and one in the Windows job.
- `crates/cache/src/core/thumbnail.rs` — tests only: a Windows grandchild-reaping
  test (§6).

## 5. Explicit non-change scope

- Production code. If a test shows Windows does **not** reap the grandchild,
  **stop and report** with the evidence — the fix is a separate decision.
- `msrv.yaml` (Linux already runs everything).
- The sidecar's own `process_tree_reaping` test — a different code path, already
  run on both platforms.

## 6. Required implementation

1. **CI:** in `native-smoke.yaml`, add `cargo test -p arama-cache --locked` to the
   macOS job and to the Windows job, next to their existing `cargo test` steps.
   Same shell, same environment conventions as the steps around them.
2. **A Windows grandchild test**, the counterpart of
   `killing_the_group_also_reaps_a_grandchild_it_spawned`, and with the same
   shape: a direct child starts a grandchild, which appends to a heartbeat file
   inside a temporary directory on a short interval. The test runs the direct
   child through `run_command_with_timeout` with a short timeout, then reads the
   file's **length** twice, a beat apart. Equal lengths mean the grandchild
   died. How the grandchild is started (`cmd`, PowerShell) is your choice;
   justify it, and make it a grandchild — a process the direct child spawned —
   not the direct child itself.
3. **The test must be able to fail.** Before trusting it, show once that it
   *does* fail when the grandchild survives — for example by spawning the
   grandchild outside the group (detached) in a throwaway copy of the test. Do
   not commit that variant; report its output.

## 7. Required tests

- The new Windows test, passing on the Windows runner.
- Its failing variant, shown failing once (§6.3) — on the runner, through a
  temporary push the architect makes, or reported as not run with the reason.
- All of `arama-cache`'s tests passing on macOS and on Windows in CI, the three
  existing `#[cfg(windows)]` tests included.
- Locally (Linux): `cargo test -p arama-cache --locked`, `clippy --all-targets
  -D warnings`, `fmt --check`.

## 8. Required documentation updates

None in the user docs. `docs/src/dev/testing.md`: one line, under its CI
description if it has one, saying `arama-cache` runs on all three platforms.

## 9. Acceptance criteria

- Both CI steps present; the macOS and Windows runs show `arama-cache`'s test
  count, with zero failures.
- The Windows grandchild test passes and was shown able to fail.
- No production change — or, if Windows does not reap, a stop-and-report instead.

## 10. Prohibited shortcuts

- Asserting only that the call returned quickly (what the existing timeout test
  does).
- Comparing timestamps instead of a length (review 162's macOS lesson applies on
  Windows too).
- `#[ignore]` on the new test, or limiting the CI step to some tests by filter.
- Any file written outside a temporary directory the test creates.

## 11. Relevant code or module boundaries

`crates/cache/src/core/thumbnail.rs`'s test module, and
`.github/workflows/native-smoke.yaml`'s macOS and Windows jobs.

## 12. Compatibility constraints

`macos-latest` and `windows-latest`, as used now. No new dependency.

## 13. Security constraints

The test processes run only on the CI runner and the developer's machine, write
only to their temporary directory, and are killed by the test. No network.

## 14. Known risks

- **The macOS and Windows jobs get slower** by one crate's test build. That is
  small, since `arama-cache` is already compiled for `arama`. Report the time the
  step adds.
- **Windows may not reap.** If `command-group`'s job object does not take the
  grandchild down, that is a real defect T061 shipped. Stop and report (§5).
- **Windows-only code cannot be run here.** The runner is the only proof; the
  architect pushes and reads the logs.

## 15. Required evidence

### 15.1 Evidence plan

| Question | Level | Evidence |
|---|---|---|
| Do `arama-cache`'s tests run on macOS and Windows? | machine, CI | both jobs' logs: the step, its test count, 0 failed |
| Does Windows reap a grandchild? | logic, on the Windows runner | the new test passing |
| Can that test fail? | logic | its detached-grandchild variant failing once |
| Does the macOS heartbeat test mean something? | logic, on the macOS runner | `killing_the_group_also_reaps_a_grandchild_it_spawned` passing there |

No GUI involved.

### 15.2 Deliverables

The rows above, observed, and the commit(s).

## 16. Review-request format

A package under `.git-exclude/review-request/` (working copy, not in the
repository) with a `## Entry point` section: the commit(s), the `git show`
commands, and full repo-relative paths, one per line. Report observed output; a
check not run is recorded as not run. You cannot push: report the commits, and
the architect pushes.

## 17. Handoff checklist

| Required item | Covered in |
|---|---|
| Task title | Heading |
| Purpose | §1 |
| Background | §2 |
| Applicable RFC and requirements | §3 |
| Change scope | §4 |
| Explicit non-change scope | §5 |
| Required implementation | §6 |
| Required tests | §7 |
| Required documentation updates | §8 |
| Acceptance criteria | §9 |
| Prohibited shortcuts | §10 |
| Relevant code or module boundaries | §11 |
| Compatibility constraints | §12 |
| Security constraints | §13 |
| Known risks | §14 |
| Required evidence | §15 |
| Required review-request format | §16 |
