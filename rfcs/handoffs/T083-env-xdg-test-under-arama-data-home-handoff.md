> **AUTHORIZED 2026-11-17 — owner:** *"Authorized."* (the follow-up from review
> 184). Written 2026-11-17 by the architect.
>
> **Independent of every open task.**

# T083 Handoff — the XDG test passes under the isolation every run uses

## 1. Purpose

Every run of arama, including `cargo test`, is to set `ARAMA_DATA_HOME` into
`.git-exclude/tmp/` (handoffs README). One test fails exactly when that is done:
`linux_local_and_cache_dirs_respect_xdg_overrides` (`env/src/dir.rs`). A test
that fails under the project's own required isolation pushes people to drop the
isolation. Make the test independent of the variable.

## 2. Background

- Observed by the architect, 2026-11-17 (review 184): with `ARAMA_DATA_HOME` set,
  `cargo test --workspace` fails at `env/src/dir.rs:245`. Without it, the test
  passes.
- **Why:** `local_dir()` and `cache_dir()` read `ARAMA_DATA_HOME` first. When it
  is set, they return paths under it, and the `XDG_*` values the test sets are
  never consulted. The test saves and restores `XDG_DATA_HOME` and
  `XDG_CACHE_HOME`, but not `ARAMA_DATA_HOME`.
- The test's own doc comment says it is the only test in that binary that mutates
  the real process environment, and why that is safe (Task 023). That reasoning
  still holds. This task only widens what it saves and restores.

## 3. Applicable RFC and requirements

RFC 041 (application data locations); the handoffs README rule *"Nothing is
created in the home directory"*.

## 4. Change scope

`env/src/dir.rs`, that one test (and its doc comment, if it lists the variables).

## 5. Explicit non-change scope

All production code in `env/src/dir.rs`; every other test. No new dependency and
no test-harness crate.

## 6. Required implementation

Inside the test, before calling `local_dir()` and `cache_dir()`: save
`ARAMA_DATA_HOME` and remove it. Restore it, with the two `XDG_*` values, **before
the assertions**, as the test already does for those, so a failing assertion
cannot leave the environment changed. Use the existing constant
`DATA_HOME_ENV_VAR`, not a string literal.

## 7. Required tests

- **Both ways, observed:** `cargo test -p arama-env --locked`, once with
  `ARAMA_DATA_HOME` set to a folder under `.git-exclude/tmp/` and once without.
  Both pass. Also show that the **unchanged** test fails with it set, which is
  the bug this fixes.
- Gates: `fmt --check`, `clippy --all-targets -D warnings`, `cargo test --workspace`
  with `ARAMA_DATA_HOME` and the four `XDG_*` homes set into `.git-exclude/tmp/`,
  all `--locked`. Report that nothing appeared under the home directory's `arama`
  folders.

## 8. Required documentation updates

None.

## 9. Acceptance criteria

- The test passes with and without `ARAMA_DATA_HOME`; the unchanged test was shown
  failing with it.
- The variable is restored before any assertion; no production change.
- All gates green.

## 10. Prohibited shortcuts

- `#[ignore]`, or skipping the test when the variable is set.
- Changing `local_dir()`/`cache_dir()` precedence.
- Running the workspace tests without the isolation "to make it pass".

## 11. Module boundaries

`env/src/dir.rs` tests.

## 12. Compatibility constraints

Linux-only test (`#[cfg(target_os = "linux")]`); the macOS and Windows tests are
untouched.

## 13. Security constraints

None beyond the isolation rule itself.

## 14. Known risks

- **Process-wide environment.** The doc comment's single-mutator argument must
  stay true. Add no second mutating test.

## 15. Required evidence

### 15.1 Evidence plan

| Question | Level | Evidence |
|---|---|---|
| Does the test pass under the required isolation? | logic | `cargo test -p arama-env`, with the variable set |
| Was it really the bug? | logic | the unchanged test failing with the variable set |
| Anything else affected? | machine | the gates, isolated |

No GUI involved.

### 15.2 Deliverables

The rows above, observed, and the commit.

## 16. Review-request format

A package under `.git-exclude/review-request/` with a `## Entry point` section:
the commit, the `git show` command, and full repo-relative paths, one per line.
Report observed output; a check not run is recorded as not run. You cannot push:
report the commit, and the architect pushes.

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
