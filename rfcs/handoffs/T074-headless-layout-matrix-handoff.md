> **NOT AUTHORIZED.** Written 2026-11-14 by the architect, the day the owner
> accepted [RFC 053](../proposed/053-headless-layout-matrix.md) (*"Accepted."*).
> Accepting an RFC does not authorize its task. Do not start until a grant
> appears in this banner.
>
> **Order: after T070 (snora 0.52 → 0.57), before T063.** From snora 0.55,
> controls grow with the text size, so a matrix measured on 0.52 would be
> re-measured on 0.57. T063 (text size) then proves its layout with this
> matrix instead of by capture (RFC 053 §4).

# T074 Handoff — the headless layout matrix, and a minimum window size

## 1. Purpose

Implement RFC 053: a headless test that renders every screen state at every
combination of text size, language, theme and window size, and checks that
every label is fully visible and nothing overflows — on every CI push, on
Linux, macOS and Windows. And make the window keep the promise the matrix
tests: a minimum size of **760 × 520** logical px of content, and a launch size
that fits the smallest laptop screen.

## 2. Background

Read RFC 053 in full first. The short version:

- Every layout problem so far (the Japanese footer, T055; the Store
  screenshots, T053/T057) was found by capture, one combination at a time.
  RFC 052's *"Larger, both locales, narrow window"* alone is more combinations
  than any capture plan can cover.
- `iced_test::Simulator` (0.14.0) can render `App::view()` at a given size with
  a given `iced::Settings` and report each text's `bounds` and `visible_bounds`.
  **Whether that detects a label overflowing a button that does not clip is not
  known** — Phase 0 finds out.
- arama sets **no** window size today (`App::start`, `app/src/core.rs`): no
  minimum, and it opens at the toolkit's 1024 × 768 — taller than the 535 px of
  content a 1366 × 768 screen at 125 % leaves.
- **The owner's decisions** (RFC 053 §7): minimum **760 × 520** (half of a
  1920 × 1080 screen at 125 %, so arama fits beside a file manager); Japanese
  font **(a)** — installed on the Linux CI runner only, nothing added to any
  release asset.

## 3. Applicable RFC and requirements

- [RFC 053](../proposed/053-headless-layout-matrix.md), all of it.
- [RFC 052](../proposed/052-text-size-and-reach.md) §2.1 — the three text sizes.
- The owner's GUI rules (`.git-exclude/rules/project-instructions-gui-common.md`,
  working copy): *Test Optimization Rules* — layout properties are checked
  headlessly, on every CI run, across every text size, language, theme and
  window size.
- `rfcs/handoffs/README.md` — the evidence-plan rule, and **nothing created in
  the home directory**.

## 4. Change scope

- `app/src/core.rs` — `App::start`: `min_size`, the launch size, the title and the Linux application id.
- `app/src/core/tests.rs` — declares the new module.
- `app/src/core/tests/layout_matrix.rs` (new) — the matrix.
- `app/src/core/tests/layout_probe.rs` (new) — Phase 0's probe (§6.1). It stays:
  the planted defects are the matrix's proof that it can fail.
- `.github/workflows/msrv.yaml` — the Japanese font step (§6.4).
- `docs/src/dev/testing.md` — a section on the matrix.

## 5. Explicit non-change scope

- **The layout itself.** Where the matrix finds a combination that does not fit,
  it is recorded (§6.5), not fixed here. Fixing the layout at the narrow width
  belongs to T063 (RFC 053 §2.2: *"fix the layout, not the size"*).
- **No bundled font.** Option (b) was not chosen. Nothing is added to
  `App::settings()`'s `fonts`, and no font file enters the repository.
- **No pixel-snapshot reference store** (RFC 053 §5 — `matches_image` passes on
  a missing reference).
- **No new workflow.** The matrix runs inside the existing test steps.
- Contrast (RFC 053 §2.3).

## 6. Required implementation

**Two deliveries.** Phase 0 first, as its own review-request package; the
architect reviews it, and RFC 053 §2.3's claims are fixed by what it shows.
Then the matrix.

### 6.1 Phase 0 — the probe (delivery 1)

Each item shown **failing on a planted defect** before anything is trusted to
pass:

1. **Clipping:** a label in a clipping container too small for it.
   `visible_bounds` must report less than `bounds`.
2. **Non-clipping overflow:** a label wider than a fixed-width button that does
   not clip — the Japanese-footer shape. Find what reports it: a text's `bounds`
   against its container's, a layout-tree walk, or `Simulator::snapshot` with a
   pixel check at the container's edge. **If nothing does, say so** — the
   property is then recorded as not checked, never claimed.
3. **Fonts:** one Japanese label measured with the Japanese font present and
   absent. The widths must differ; if they do not, the font is not in use.
4. **Which font draws kanji** (RFC 053 §2.4's finding): on each of the three CI
   runners, the font face selected for one kanji and one kana in the same
   Japanese label, through the `cosmic-text` iced already uses
   (`iced_graphics::text::cosmic_text` re-exports it — **do not add a second
   copy to the lockfile**). Recorded, not asserted.
5. **Which screen states can be built headless** (RFC 053 §2.1's list), and
   which need state constructed directly on `App`.
6. **Run time** of one combination × all states, to project the full matrix
   against the 60 s budget.

Items 3–4 need the CI runners, which you cannot reach. Put the probe in the
default test suite and have it print its findings: `native-smoke.yaml`'s default
`cargo test -p arama --lib --locked -- --nocapture` already shows them for macOS
and Windows, and the architect pushes and reads them. For Linux, run it on the
development machine, which already has Noto CJK installed; the CI value comes
from the log line of §6.4. Report each runner's result as observed or not run.

### 6.2 The minimum and launch size

In `App::start`: `iced::window::Settings { min_size: Some(760 × 520), size: …,
.. }` through `.window(…)` (iced 0.14.0, `application.rs:253`). The launch size
must fit inside 1093 × 535 logical px — **1024 × 520** unless Phase 0 shows a
reason otherwise. One named constant per value, with the reason beside it
(RFC 053 §2.2). Both are logical px of the content area.

**Added 2026-11-14 (review 165, P1) — the window's name, in the same
`window::Settings`.** arama sets no title and no application id today. iced's
default title is `format!("{title} - Iced")` (`iced_program` 0.14.0,
`src/lib.rs:90`), so every platform shows **"Arama - Iced"** in the taskbar,
the window switcher and the title bar. On Linux, a window with no app id cannot
be matched to `packaging/linux/arama.desktop`: users get a generic icon and no
grouping with the launcher. Set:

- the title, through `.title(…)`, to **`arama`** — the product name exactly as
  `arama.desktop`'s `Name=` spells it, in every locale;
- `platform_specific.application_id` to **`arama`** on Linux — the desktop
  file's basename, as iced's own documentation for that field advises
  (`iced_core` 0.14.0, `window/settings/linux.rs`). On X11, winit sets
  `WM_CLASS` from the same value.

### 6.3 The matrix (delivery 2)

As RFC 053 §2.1: text size (Default only until T063 lands, the dimension wired)
× `en`, `ja` × the four presets in `crates/theme/src/lib.rs` × **760 × 520,
1280 × 720, 1920 × 1080**, for every state Phase 0 can build. Properties: every
one Phase 0 showed it can make fail, and only those. The Simulator gets
`App::settings()` — the real fonts and text size, not a test copy.

**Isolation:** constructing `App` resolves and writes data locations. Every
matrix test takes `ARAMA_DATA_HOME_TEST_LOCK` and points `ARAMA_DATA_HOME` at a
temporary directory (`app/src/core/tests.rs` already does this — reuse it).
Nothing may be written in the home directory.

**A failure names its combination** (RFC 053 §2.6): screen state, the four
values, the text's `content`, both rectangles.

### 6.4 The Japanese font on CI

- **Linux (`msrv.yaml`):** before `cargo test`, `sudo apt-get install -y
  fonts-noto-cjk`, then print its version (`dpkg-query -W fonts-noto-cjk`) into
  the log. Not pinned to an exact package version — that breaks when the runner
  image updates; the log records what was measured.
- **macOS, Windows (`native-smoke.yaml`):** install nothing — those runners'
  system fonts are what those users have. Phase 0 confirms each runner has a
  Japanese face; if one does not, stop and report.
- **Every runner:** the matrix **fails** — never skips — if no face covering a
  kana is in the font database. A matrix that silently measured fallback boxes
  would pass for the wrong reason.

### 6.5 What does not fit today

If a combination fails at 760 × 520 today, it goes in one list in
`layout_matrix.rs`, `KNOWN_LAYOUT_DEFECTS`: state, combination, the failing
text, and the task that owns the fix (T063 for the narrow width). **Each entry is
asserted to still fail** — when a fix lands, its entry makes the test fail until
it is removed, so the list cannot go stale. Anything else that fails is a test
failure.

## 7. Required tests

- Phase 0's items 1–3, each shown failing on its planted defect, then passing.
- The matrix, every combination of §6.3.
- `min_size` and launch size: a unit test reads `App`'s window settings and
  asserts both values — the constants are not only defined but used.
- Title and application id: the same test asserts both are `arama`, and that
  the application id equals `arama.desktop`'s basename, read from
  `packaging/linux/` — so renaming one without the other fails.
- The kana-face check failing when the font database has no Japanese face (a
  font database built without one).
- `cargo test --workspace --locked`, `clippy --all-targets -D warnings`,
  `fmt --check`.

## 8. Required documentation updates

`docs/src/dev/testing.md`: *"Layout matrix"* — what it checks, which properties
it does **not** check (from Phase 0), how to run one combination by filter, the
Japanese font a Linux machine needs for the matrix to run (Noto CJK), and what
`KNOWN_LAYOUT_DEFECTS` means.

## 9. Acceptance criteria

RFC 053 §6, and:

- Phase 0 reviewed and accepted before the matrix is delivered.
- Matrix under 60 s on the Linux runner, or split (not trimmed) with the
  numbers reported.
- `min_size` 760 × 520 and a launch size inside 1093 × 535, both tested.
- Title `arama` and Linux application id `arama`, both tested (§6.2).
- No font file in the repository; release assets unchanged in size.
- Green on all seven workflows after the architect pushes.

## 10. Prohibited shortcuts

- Claiming a property Phase 0 could not make fail.
- Skipping, rather than failing, when the Japanese font is missing.
- Trimming the matrix to meet the budget.
- Fixing layout here, or raising the minimum to make a combination pass.
- A `#[ignore]`d matrix, or one that runs on Linux only.
- Any write to the home directory; any test that constructs `App` without the
  isolation in §6.3.
- Installing anything system-wide on the development machine (a font package
  included). It already has a Japanese font; if a check says otherwise, stop and
  report.

## 11. Relevant code or module boundaries

`app/src/core.rs` (`App::start` only), `app/src/core/tests.rs` and its new
submodules, `.github/workflows/msrv.yaml`, `docs/src/dev/testing.md`. The
theme, i18n and snora crates are read, not changed.

## 12. Compatibility constraints

- Rust baseline from `Cargo.toml` (`msrv.yaml` runs it).
- iced 0.14.0 / iced_test 0.14.0 / cosmic-text 0.15.0, as locked. No new
  dependency beyond what is already in the lockfile.
- Wayland and X11: `min_size` is a request the compositor may honour
  differently; it is tested as a setting, not by resizing a real window.

## 13. Security constraints

- The repository is public: no home-directory path, user name or email address
  in code, docs, test output committed, or the package.
- The apt step installs one distribution package by name from the runner's own
  archive; nothing is downloaded from elsewhere.

## 14. Known risks

- **Non-clipping overflow may be undetectable** headlessly. Then it is recorded
  as a gap, and real-UI captures keep covering it (RFC 053 §2.3).
- **The headless renderer may not load system fonts** the way the real app does.
  Phase 0 item 3 catches this.
- **The Windows runner may lack a Japanese face.** Stop and report; do not
  install fonts there without the architect.
- **Many combinations may fail at 760 today.** That is the expected result, not
  a failure of the task — §6.5.

## 15. Required evidence

### 15.1 Evidence plan

| Question | Level | Evidence |
|---|---|---|
| Can the harness see clipping, overflow, and font use? | headless | Phase 0's planted defects, failing then passing |
| Which font draws kanji on each platform? | headless, on CI | Phase 0 item 4's output from all three runners |
| Does every combination keep every label visible? | headless | the matrix, its run time, `KNOWN_LAYOUT_DEFECTS` |
| Are the minimum and launch sizes set? | logic | the unit test of §7 |
| Does the window carry arama's name and app id? | logic, then machine | the unit test of §7; then niri's window list showing `title` and `app_id` both `arama`, from a session started through `scripts/gui-session.sh` — no capture needed |
| Is Japanese measured with a real font everywhere? | headless, on CI | the kana-face check; `dpkg-query` line in the Linux log |
| Does the real window honour the minimum? | real UI | **none in this task** — one capture at 760 × 520 belongs to T063's evidence, through `scripts/gui-session.sh` |

### 15.2 Deliverables

The rows above, observed, and the commits — for each of the two deliveries.

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
