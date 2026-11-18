> **AUTHORIZED 2026-11-18 — owner:** *"Authorized."* Written 2026-11-18 by the architect, as item 2 of the
> 0.44.0 plan (`.git-exclude/schedule/README.md`, owner: *"Authorized."*) under
> [RFC 054](../proposed/054-keyboard-reach.md) (owner: *"Accepted."*). Neither was
> authorization of this task; this banner is.
>
> **First of three keyboard tasks** (T085 → T086 → T087). Independent of T088 and
> T089.

# T085 Handoff — snora 0.65: Tab and F6 move real focus, and the Phase 0 questions

## 1. Purpose

Move arama from snora 0.57 to 0.65 and use what 0.65 gives keyboard users at
once: **Tab moves between controls, and F6 moves focus into the next region**
instead of a ring around it. Then answer RFC 054's six Phase 0 questions, which
decide how T086 and T087 are built. No widget is replaced here.

## 2. Background

- **Today Tab does nothing, and F6 moves a ring, not focus** (RFC 054, Summary).
  The footer says *"F6 switches panels"*; after F6, Enter acts on nothing.
- **Measured by the architect, 2026-11-18,** in a scratch worktree (removed
  since): `snora = { version = "0.65", … }` in `Cargo.toml:42`, then
  `cargo update -p snora`. **Exactly six packages moved**, each 0.57.0 → 0.65.0:
  `snora`, `snora-core`, `snora-design`, `snora-interaction`, `snora-style`,
  `snora-widgets`. Nothing else. snora 0.65 declares `rust-version = "1.88"` and
  `iced = "0.14"`, so neither the baseline (1.94) nor iced moves.
- **`cargo check --workspace --all-targets` then fails at the F6 sites only**:
  7 errors in the `arama` library and 8 more in its tests, all
  `FocusZone`, `ZonePresence` or `next_zone`, at:
  - `app/src/core.rs:134`, `:442`, `:456`, `:457`
  - `app/src/core/view.rs:8`
  - `app/src/core/update/keyboard.rs:3`, `:17`, `:57`
  - `app/src/core/tests.rs:526`, `:538`, `:549`, `:556`, `:841`, `:853`, `:865`

  arama calls none of the eight functions that gained a `direction` in 0.64, nor
  `row_dir`, `Shortcuts::routes`, or the removed toast-hover items (checked by
  `grep`, and confirmed by the build reaching only these errors).
- **snora's replacements:**
  - F6: `snora::keyboard::cycle_zones(key, modifiers)` then
    `snora::keyboard::focus_zone(cycle)`, which returns a `Task`. The app keeps
    no zone state (snora's 0.60 → 0.61 guide).
  - Tab: `snora::keyboard::traverse(key, modifiers)`, which returns
    `Option<Task>` (snora 0.65, `src/keyboard.rs`).
- **Colours change without code changes:** 0.62 changed 17 state colours and a
  notice's action colour (snora's contrast fixes). arama draws its progress bars
  with iced's own `progress_bar`, not snora's, so 0.64's new progress bar does
  not reach it. Q5 confirms both.
- snora's guides, one per step:
  - <https://nabbisen.github.io/snora/guides/migrations.html>
  - <https://nabbisen.github.io/snora/guides/migration-0.57-to-0.58.html>, and
    so on to `migration-0.64-to-0.65.html`
  - <https://nabbisen.github.io/snora/guides/keyboard.html>

## 3. Applicable RFC and requirements

[RFC 054](../proposed/054-keyboard-reach.md) §1 (principles), §2, §3.1, §4
(Phase 0), §7. [RFC 044](../done/044-keyboard-operability.md): RFC 054 §3.1
replaces its §2.2 ring. The handoffs README rules: isolation; binaries only
through `cargo test`, `cargo run` or `scripts/gui-session.sh`; probes only in a
scratch worktree with its own `CARGO_TARGET_DIR`; toolchains only through a
project-local `RUSTUP_HOME`.

## 4. Change scope

- `Cargo.toml` (the `snora` line), `Cargo.lock`
- `app/src/core.rs`
- `app/src/core/view.rs`
- `app/src/core/update/keyboard.rs`
- `app/src/core/message.rs` (doc comments that name the removed types)
- `app/src/core/tests.rs`
- `crates/theme/src/lib.rs` (`focus_ring_style`, once nothing uses it)
- `docs/src/dev/testing.md`, wherever it describes the removed ring or F6 test
- `CHANGELOG.md`, under `[Unreleased]`

## 5. Explicit non-change scope

- **No widget swap.** Buttons, checkboxes, sliders, the `pick_list`, the
  settings choices and every `mouse_area` stay as they are: that is T086 and
  T087.
- No new focus ids, no dialog initial focus (T086).
- iced's `advanced` feature is **not** enabled in the commit. Q1 probes it in a
  scratch worktree only.
- Any dependency other than the six snora crates.

## 6. Required implementation

1. **The upgrade.** `snora = { version = "0.65", … }`, then
   `cargo update -p snora`. **List every package that moves** and compare it
   with §2. **Stop if anything differs.**
2. **One pure routing function.** In `app/src/core/update/keyboard.rs`, a pure
   function from `(key, modifiers)` to what the key means. For example:

   ```rust
   enum KeyRoute { Traverse, Zone(Cycle) }
   ```

   Tab is tested first, then F6. `handle_key_pressed` maps the route to
   snora's task: `traverse(...)` for Tab, `focus_zone(cycle)` for F6.
   Everything else returns `Task::none()`, as today. With a dialog or menu open,
   do nothing special: snora contains focus (RFC 054 §2).
3. **Remove the ring and its state:**
   - the `focus_zone` and `focus_visible` fields;
   - `App::zone_presence()`, `zone_after_cycle`, and `is_zone_focused`;
   - the three `arama_theme::focus_ring_style(...)` call sites in
     `app/src/core/view.rs`, and the function itself if nothing else uses it.

   Delete them; do not keep them "for later". Replace their tests with tests of
   §6.2's function: Tab, Shift+Tab, F6, Shift+F6; Ctrl+Tab, Alt+Tab and
   Logo+Tab are not Tab; other keys route nowhere.
4. **The RFC 044 keyboard-baseline test** (`app/src/core/tests.rs`, from line
   ~465, *"Phase 0.1 keyboard baseline"*): update it to the new behaviour, or
   remove it if §6.3's tests cover all of it. Say which, and why.
5. **Phase 0: answer Q1 to Q6** (RFC 054 §4). Use a scratch worktree with its
   own `CARGO_TARGET_DIR` for any probe code. **Nothing from a probe is
   committed.** For each question, report the answer, the evidence (code
   reference, output, or capture), and what it means for T086 or T087.
   - **Q1:** can arama tell which control holds focus? Try iced's `advanced`
     feature (`iced::advanced::widget::operation::focusable::find_focused`, or
     whatever 0.14 names it), and anything snora 0.65 exposes. Report the API,
     what it returns, whether it reaches a `snora::control`'s `.id()`, and the
     time per call on a screen with about 1,000 widgets. Use a plant in the
     probe worktree if needed.
   - **Q2:** after this task, with the ring gone, which regions does F6 stop on?
     The footer has only an iced slider, which is not focusable. snora says F6
     goes to the first zone that has a control. Observe it, and say whether the
     footer is skipped until T086.
   - **Q3:** can the folder tree (`iced-swdir-tree` 0.9.3, its
     `DirectoryTree::handle_key`) get one focus target from arama's side, without
     a click on a row turning into a press of a wrapping control? If not
     cleanly, describe what the tree crate would need. It is the owner's crate,
     so that is a note for the owner, not a change here.
   - **Q4:** does snora draw its focus ring on a `snora::control` given a host
     `.style()` such as `arama_theme::secondary`? Answer on two presets.
   - **Q5:** does anything look different on 0.65? Run the layout matrix (RFC
     053) on `origin/main` and on your commit. List every capture that changed,
     and why.
   - **Q6:** can arama know a gallery item's on-screen position without a pointer
     event (for a keyboard-opened context menu)?
6. **Contrast.** Run the existing contrast tests (the T064 side-bar active fill,
   the disabled-button test in `crates/theme/src/lib.rs`). Report each printed
   ratio **before and after** the upgrade, per preset.
7. **Observe in a real window,** through `scripts/gui-session.sh`, with
   `xdotool key` only (no clicks). Fresh isolated profile, the small fixture
   gallery:
   - Tab from a fresh start: where focus lands, and that the ring is visible.
   - F6 forward and back through the regions; then Enter on what F6 reached.
   - With Settings open: Tab stays inside the dialog. Escape closes it.
   - Shift+Tab, and Tab past the last stop (snora: focus leaves, and the next
     Tab starts from the top).
8. **`CHANGELOG.md` `[Unreleased]`:** Tab now moves between the controls that
   can take focus; F6 moves focus into the next panel; snora 0.65's colour
   corrections. Name only what this task changes. T086 and T087 add the rest.

## 7. Required tests

- §6.3's routing tests (Tier 1).
- Gates, all `--locked`: `fmt --check`, `check`, `clippy --all-targets -D warnings`,
  `test --workspace` (with `ARAMA_DATA_HOME` and the four `XDG_*` homes in
  `.git-exclude/tmp/`), `audit --deny warnings`, T066's script, the untranslated
  strings check.
- `cargo +1.94 check --workspace --locked`, with
  `RUSTUP_HOME="$PWD/.git-exclude/tmp/rustup-1.94"` (check
  `rustup toolchain list` under it first).
- The layout matrix (Q5).

## 8. Required documentation updates

`CHANGELOG.md` (§6.8). `docs/src/dev/testing.md` where it describes the removed
ring or the F6 test. Nothing in `docs/src/users/`: the user-facing keyboard
section comes with T087, when there is a complete story to tell.

## 9. Acceptance criteria

- Exactly the six snora crates moved; the baseline is unchanged.
- Tab and F6 move real focus in a real window (§6.7). Nothing outside an open
  dialog reacts while it is open.
- No zone state and no region ring remain in the code.
- Q1–Q6 each answered with evidence.
- Contrast ratios reported before and after; every changed matrix capture
  explained.
- All gates green, including the 1.94 check.

## 10. Prohibited shortcuts

- Replacing any widget (T086, T087).
- Keeping `focus_zone`, `focus_visible` or `focus_ring_style` unused, or behind
  `#[allow(dead_code)]`.
- Committing probe code, or enabling iced's `advanced` feature in the commit.
- A bare `cargo update`.
- Running any arama binary except through `cargo test`, `cargo run` or
  `scripts/gui-session.sh`; never by name or glob from `target/`.
- Inferring §6.7 from the code instead of observing it.

## 11. Module boundaries

`app/src/core` (keyboard routing, view, state); `crates/theme` (one function
removed). No UI crate changes.

## 12. Compatibility constraints

Rust 1.94; iced 0.14 unchanged. Linux is observed here; macOS and Windows build
and run their native smoke in CI (the architect watches them after pushing).
F6 on macOS may need the Fn key, depending on the keyboard: say so if the probe
shows it, but do not add a binding for it here.

## 13. Security constraints

None new. The usual isolation for every run: nothing written outside
`.git-exclude/tmp/`.

## 14. Known risks

- **The footer may have no F6 stop until T086** (Q2). This is expected; report
  it, do not work around it.
- **F6 from a context menu:** snora says it cannot tell which zone a free-standing
  menu means (`snora::keyboard` docs). Observe what happens and report it.
- **Matrix captures move** with 0.62's colours. Each one needs a reason, not a
  blanket re-baseline.

## 15. Required evidence

### 15.1 Evidence plan

| Question | Level | Evidence |
|---|---|---|
| Did exactly the measured set move? | machine | the lockfile diff vs §2 |
| Do Tab and F6 route correctly? | unit | §6.3's tests |
| Do they move real focus? | real UI | §6.7 observed; 2 captures: F6's ring on a control, and Tab inside the Settings dialog |
| What does 0.65 change visually? | headless | the matrix before/after (Q5), with reasons |
| Phase 0 | probe | Q1–Q6 answers with evidence |
| Did contrast move? | unit | the ratios before/after |

### 15.2 Deliverables

The commit, and the rows above, observed.

## 16. Review-request format

A package under `.git-exclude/review-request/` with a `## Entry point` section:
the commit, the `git show` command, and full repo-relative paths, one per line,
to the evidence files. Report observed output; a check not run is recorded as
not run. You cannot push: report the commit, and the architect pushes.

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
