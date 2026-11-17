> **NOT AUTHORIZED — awaiting the owner.** Written 2026-11-18 by the architect,
> as part of the 0.43.0 release plan (`.git-exclude/schedule/README.md`).
>
> **Starts after the architect's release-preparation commit** (version 0.43.0,
> `CHANGELOG.md`, RFC moves). The architect names that commit in this banner
> when it lands. Every pass below runs **on that commit**.
> **Cutting the release (tag, publish) is not part of this task.** It is the
> owner's and the architect's.

# T084 Handoff — release-candidate evidence for 0.43.0

## 1. Purpose

`docs/src/dev/release.md` requires checks that run only once per release, on the
release commit, with observed output: the gates, the external-FFmpeg contract,
a local archive sanity check, **a real model download**, and **an upgrade from
the previous release**. Produce that evidence for 0.43.0, so the owner can
authorize the tag on facts.

## 2. Background

- **0.42.0** was tagged 2026-09-07. `main` is 116 commits past it.
- **This is the first release with the two expensive passes** (added to
  `release.md` 2026-11-14).
- **The upgrade pass matters more than usual:**
  - T067 moved the cache schema one way (`localcache` 0.21 → 0.26.2).
  - T063 added a text-size setting, and T074 window settings.
  - The schedule's *"For the 0.43.0 release notes — the downgrade warning,
    corrected"* records what happens on a downgrade: the older build refuses a
    v6 cache cleanly, then indexes silently to nothing and cannot clear it.
    The release notes must say what a downgrading user does. This pass checks
    that they are true.
- **0.42.0's published binary:** `arama@Linux-x64-gnu-cpu-0.42.0.tar.gz` on the
  GitHub release. It has no `SHA256SUMS` (T075 starts with 0.43.0), but GitHub
  records a per-asset `digest` (`gh release view 0.42.0 --json assets`).
- `scripts/gui-session.sh` launches any binary through `GUI_SESSION_BINARY`,
  with the isolation and the preflight refusals. Fixtures live in
  `.git-exclude/fixtures/` (`SHA256SUMS` there).

## 3. Applicable RFC and requirements

`docs/src/dev/release.md` (steps 1, 5, 6, *External FFmpeg artifact-absence
gate*, *Once-per-release expensive passes*, the checklist);
`docs/src/dev/testing.md` (release smoke); RFC 041; RFC 021 and T071 (model
download); the handoffs README rules (isolation; binaries only through
`cargo test`/`cargo run`/`gui-session.sh`; toolchains only through a
project-local `RUSTUP_HOME`).

## 4. Change scope

**No commit to the repository**, unless a pass finds a defect. In that case stop
and report it, and fix nothing in this task. Evidence goes into the package and
into `.git-exclude/tmp/t084-*/`.

## 5. Explicit non-change scope

Everything tracked: version, `CHANGELOG.md`, RFCs, workflows, code. No tag, no
release, no `workflow_dispatch` (the architect runs that).

## 6. Required implementation

1. **Gates on the release commit** (release.md step 1): `fmt --check`, `check`,
   `test --workspace`, `clippy --all-targets -D warnings`,
   `audit --deny warnings`, T066's script. All `--locked`, with tests under full
   isolation.
2. **External FFmpeg contract:** `bash scripts/check-external-ffmpeg-contract.sh`,
   then with `--archive` for the local archive from step 3 and `--binary` for a
   release build. Build it with `cargo build --release --locked`; a release build
   is not a run.
3. **Local archive sanity** (release.md steps 5–6): build `arama-0.43.0.tar.gz`
   into `.git-exclude/tmp/t084-archive/`. Show that its files sit at the root
   and that it excludes `.git/`, `.git-exclude/`, `target/` and `docs/book/`.
4. **Real model download** (expensive pass 1): run first-run setup under a fresh
   isolated profile (`ARAMA_DATA_HOME` plus the four `XDG_*` homes in
   `.git-exclude/tmp/t084-download/`), through `gui-session.sh` with the
   release build. Let it download both models. Record:
   - the elapsed time;
   - the bytes received;
   - that both pinned SHA-256 checks passed (`crates/ai/src/model/model_container/`);
   - the hosts the redirects reached, if your evidence shows them.
5. **Upgrade from 0.42.0** (expensive pass 2), in a second isolated profile
   (`.git-exclude/tmp/t084-upgrade/`):
   1. Download the 0.42.0 Linux asset into `.git-exclude/tmp/t084-upgrade/bin/`
      and check its SHA-256 against GitHub's `digest` **before** running it.
   2. Run it through `gui-session.sh` on a small gallery from
      `.git-exclude/fixtures/`, so it writes settings and a cache. Change one
      setting (theme or locale).
   3. Run the 0.43.0 release build on the same profile.
   4. Show that the setting survived, the gallery appears without indexing from
      nothing, and no error toast appears.
   5. **The downgrade, on a copy of that profile:** run 0.42.0 again and record
      exactly what the user sees. The release notes are written from that.
6. **Release smoke** (`testing.md`): fill in the
   `release-smoke-evidence-template.md` for what this machine can exercise. Mark
   the rest `not run` or `environment-dependent`.

**Captures:** the evidence plan's worst case only (§15.1). The upgrade pass's
before and after windows, and the downgrade's first screen.

## 7. Required tests

§6 is the test. A pass not run is recorded as not run, with the reason.

## 8. Required documentation updates

None. Release notes are the architect's, from your §6.5.5 observation.

## 9. Acceptance criteria

- Every §6 item observed and reported with its output, on the named release
  commit.
- The 0.42.0 binary's checksum matched before it ran.
- Nothing written outside `.git-exclude/tmp/` (show the home directory's `arama`
  folders absent afterwards, and `~/.rustup` unchanged).

## 10. Prohibited shortcuts

- Running any arama binary, including 0.42.0, except through `gui-session.sh`
  (or `cargo test`). Never by name or glob from `target/`.
- Reusing a profile between the download and upgrade passes, or seeding the
  upgrade profile with 0.43.0 data.
- Running the 0.42.0 binary before its checksum is checked.
- Inferring the downgrade behaviour from the schedule instead of observing it.

## 11. Module boundaries

None changed. Scratch only.

## 12. Compatibility constraints

Linux x64 here. macOS and Windows are covered by CI's native smoke and by the
release workflow itself. Say so rather than claim them.

## 13. Security constraints

- The 0.42.0 binary is a downloaded executable: check its checksum first, then
  run it only isolated.
- The model download goes only to the pinned URLs (HTTPS-only since T071).

## 14. Known risks

- **HuggingFace changes without notice** (it moved to a regional server in
  November 2026). A failure here is a finding, not a flake. Report it.
- **The upgrade may expose a real defect.** Stop and report it; do not work
  around it.
- **Disk:** two models (about 1 GB) per download profile, plus two builds. Clean
  up `.git-exclude/tmp/t084-*` after the package is written, except the
  captures.

## 15. Required evidence

### 15.1 Evidence plan

| Question | Level | Evidence |
|---|---|---|
| Does the release commit pass every gate? | machine | the gate outputs |
| Does the archive meet its contract, with no ffmpeg bundled? | machine | the contract script, the archive listing |
| Does a real download work today? | real system | elapsed time, bytes, checksum results |
| Does 0.42.0 → 0.43.0 keep settings and cache? | real UI | 2 captures (before, after) and the state files |
| What does a downgrading user see? | real UI | 1 capture and the observed behaviour |

### 15.2 Deliverables

The rows above, observed, in one package.

## 16. Review-request format

A package under `.git-exclude/review-request/` with a `## Entry point` section:
the release commit it ran on, and full repo-relative paths, one per line, to the
evidence files. Report observed output; a check not run is recorded as not run.

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
