> **NOT AUTHORIZED.** Written 2026-11-19 by the architect, as item 3 of the
> 0.44.0 plan, under [RFC 054](../proposed/054-keyboard-reach.md) (accepted), after
> T085 and its follow-up F1 (reviews 189, 192). Do not start until a banner here
> says so.
>
> **Second of three keyboard tasks** (T085 → **T086** → T087).

# T086 Handoff — every control is a Tab stop

## 1. Purpose

Make every control that is not part of a collection reachable and operable from
the keyboard, with a visible ring:

- the buttons and the five form controls;
- the three one-of-N settings, as radio groups;
- the About link;
- where focus starts in each dialog, and where it returns when the dialog
  closes.

Collections (the gallery, the pairs list, the media focus strip, the folder
tree) are T087.

## 2. Background

- **F1's inventory (review 192):** the explorer has 5 Tab stops, the cache page
  6, the settings page 3 (the rail only). **Setup, Confirm, Media focus and
  Similar pairs have none.** The cause is in iced's own source:
  `button::operate` never registers a focus target, so a plain
  `iced::widget::button` is never a Tab stop.
- **The sites.** Counted 2026-11-19, `button(` / `checkbox(` / `slider(` /
  `pick_list(` per file:

  | File | button | checkbox | slider | pick_list |
  |---|---|---|---|---|
  | `app/src/core/view.rs` | 2 | | | |
  | `crates/ui/layout/src/footer/thumbnail_size_slider/view.rs` | | | 1 | |
  | `crates/ui/layout/src/header/dir_nav/view.rs` | 1 | | | |
  | `crates/ui/layout/src/header/view.rs` | 1 | | | |
  | `crates/ui/main/src/core/components/setup/downloader/view.rs` | 2 | | | |
  | `crates/ui/main/src/core/views/cache_page/view.rs` | 5 | | | |
  | `crates/ui/main/src/core/views/gallery/view.rs` | 1 | | | |
  | `crates/ui/main/src/core/views/setup/view.rs` | 2 | | | |
  | `crates/ui/widgets/src/context_menu/view.rs` | 2 | | | |
  | `crates/ui/widgets/src/dialog/confirm_dialog/view.rs` | 2 | | | |
  | `crates/ui/widgets/src/dialog/media_focus_dialog/view.rs` | 5 | | | 1 |
  | `crates/ui/widgets/src/dialog/settings_dialog/tab/ai_settings/view.rs` | 7 | | | |
  | `crates/ui/widgets/src/dialog/settings_dialog/tab/file_system_settings/view.rs` | 1 | | | |
  | `crates/ui/widgets/src/dialog/settings_dialog/tab/general_settings/view.rs` | 5 | 2 | 1 | |
  | `crates/ui/widgets/src/dialog/settings_dialog/view.rs` | 1 | | | |

  Some `button(` sites are inside loops (the Language, Theme and Text size
  rows), so there are more buttons on screen than call sites. Plus one
  `mouse_area(text(…))`: the About tab's repository link.
- **snora 0.65 provides:**
  - `snora::design::control(&tokens, content)`. It takes
    `.style(fn(&Theme, button::Status) -> button::Style)`, so
    `arama_theme::{primary, ghost, secondary, danger,
    primary_readable_when_disabled}` carry over. It is a focus target **only if
    it can be pressed** (`snora-interaction-0.65.0/src/control.rs:522-528`): a
    disabled control is not a stop, by snora's rule.
  - `snora::design::{checkbox, toggler, radio_group, select, slider}`, each one
    Tab stop. A radio group is one stop, with arrows inside. A `select` is open
    when the application says so, and is closed by the one `on_close_menus`
    message the application gives the engine (snora's form module docs).
- **The ring.** snora draws it 2 px **outside** the control, on the page, in a
  colour chosen to read against the surface it names as `behind` (review 192).
  For ordinary controls `behind` is the page, and that is right. **The active
  rail item's ring is the exception**: white on a light page, 1.08:1, measured
  from T085's capture. That is snora's, reported in a draft letter. T086 does
  not work around it.
- **Settings is a page, not a dialog** (review 189; RFC 054 §3.3 corrected).
- `settings_dialog::Tab` is not public outside `arama-ui-widgets` (F1), so no
  test outside the crate can select the AI or File system tab.

## 3. Applicable RFC and requirements

[RFC 054](../proposed/054-keyboard-reach.md) §1 (principles 1, 2, 4, 5), §2,
§3.2, §3.3, §3.4, §6 (the acceptance walk, for the screens this task covers).
WCAG 2.1 SC 2.1.1, 2.4.3, 2.4.7, 1.4.11. The handoffs README rules.

## 4. Change scope

- Every file in §2's table, plus
  `crates/ui/widgets/src/dialog/settings_dialog/tab/about/view.rs`.
- The `update` and state files next to them, where a `select` needs an open flag
  or a dialog needs an initial-focus task.
- `app/src/core/update/*.rs`: initial focus when a dialog opens, the return of
  focus when it closes, and `on_close_menus` covering an open `select`.
- `crates/ui/widgets/src/dialog/settings_dialog.rs`: make `Tab` public if a test
  needs it.
- `crates/theme/src/lib.rs`, only if a style function needs a
  `snora::design::control` form.
- The tests next to each, plus the committed inventory test (§6.6).
- `CHANGELOG.md` `[Unreleased]`.

## 5. Explicit non-change scope

- **Collections:** gallery cells, similar-pairs items, the media focus strip,
  the folder tree (T087). Their `mouse_area`s stay.
- The context menu's *opening* by keyboard (T087). Its two items **do** become
  controls here, so snora's menu arrows reach them once it is open.
- The rail's ring colour (snora's; the letter is the owner's).
- No new global shortcut.
- No change to any message's meaning: a key press sends exactly what the click
  sends.

## 6. Required implementation

1. **Buttons → `snora::design::control`.** Every site in §2, with the same style
   function, the same content and the same message. Where a button is disabled
   by `on_press_maybe(None)`, keep that. The control is then correctly not a
   stop.
2. **Form controls:**
   - the two checkboxes (include images, include videos) →
     `snora::design::checkbox`;
   - the two sliders (similarity threshold, thumbnail size) →
     `snora::design::slider`, same range and step;
   - the media focus `pick_list` → `snora::design::select`, with its open state
     in the dialog's state and closed through `on_close_menus` ("close whichever
     is open").
3. **Language, Theme, Text size → `snora::design::radio_group`**, one per
   setting, same labels and messages. The *selected* look is snora's; say in the
   package how it differs from today's button row.
4. **The About link → a control styled as a link**, opening the same URL.
5. **Dialog focus (RFC 054 §3.3):**

   | Dialog | Focus on open | Note |
   |---|---|---|
   | Confirm (Settings → File system → *Cache delete*, its only use) | **Cancel** | Enter on open must never confirm. |
   | Media focus | Close | T087 moves it to the strip's current item. |
   | Similar pairs | Close | T087 moves it to the first pair. |

   Give each a stable `.id()` and return `operation::focus(id)` from the update
   that opens it. **On close, focus returns to the control that opened it.**
   Give the openers ids: the Similar pairs button, and *Cache delete*. The
   media focus dialog is opened from a gallery cell (T087), so for now record
   which opener has no id yet. If the opener is gone, focus goes nowhere
   (snora's rule).
6. **The inventory, committed.** Turn F1's `Inventory` probe into a committed
   test. It goes under `app/src/core/tests/` and uses `iced_test`'s re-exports,
   as F1 did. For each screen and overlay it asserts the expected focus targets
   in Tab order. Assert at least the count, and an id where one is set.
   - **Every** screen must then have stops: setup, explorer (aside open and
     closed), cache, settings (each tab), confirm, media focus, similar pairs.
   - This is the regression guard: a plain `button` added later shows up as a
     missing stop.
7. **The ring, measured as drawn.** For each control kind on its real surface
   (page, card, dialog), on all four presets: compute the colour snora draws
   (`RingConfig::color`'s rule, or a headless render read back) and its contrast
   with what is around the ring. Report any pair under 3:1. If one is arama's
   (a host style that changes what is behind a control), fix it here. If it is
   snora's, report it. Do not override snora's ring.
8. **The real-window walk** (RFC 054 §6 steps 1, 2, 6, 7 and 8; steps 3–5 are
   T087's). `scripts/gui-session.sh`, `xdotool key` only, a fresh isolated
   profile:
   - Start with one harmless key, then capture after each of the first three
     Tabs. This settles review 192's open question about the first Tab.
   - The walk covers:
     - setup by keyboard (download or skip);
     - choose a fixture folder with the folder field;
     - Settings: language, theme and text size by arrows; toggle *include
       videos*; move the similarity slider;
     - Cache page: filter, and use each of its actions;
     - Settings → File system → *Cache delete*: the confirm opens on **Cancel**,
       Enter cancels, and focus is back on *Cache delete*. Then open it again, Tab
       to *Delete* and confirm;
     - F6 through every region, now including the footer.
   - Note anything that does not respond.

## 7. Required tests

- §6.6's inventory test.
- Tier 1, per dialog: the task returned on open targets the expected id, and on
  close it targets the opener.
- The existing tests, updated where a widget changed type. A test that only
  checked the iced widget's type is replaced, not deleted.
- Gates, all `--locked`: `fmt`, `check`, `clippy -D warnings`, isolated
  `test --workspace`, `audit --deny warnings`, T066's script, untranslated
  strings, the 1.94 check with the project-local `RUSTUP_HOME`.
- The layout matrix (RFC 053) passes. Explain any geometry change, because a
  snora control's 24 px minimum target can move things.

## 8. Required documentation updates

- `CHANGELOG.md` `[Unreleased]`: every button and setting can now be reached
  with Tab and used with Enter, Space or the arrows; dialogs open on a safe
  choice; the look of the checkboxes and choices in Settings.
- `docs/src/users/settings.md`, where it describes how a setting is chosen.

## 9. Acceptance criteria

- No `iced::widget::button`, `checkbox`, `slider` or `pick_list` remains in
  `app/src` or `crates/ui`, outside T087's collections and tests.
- The committed inventory test shows stops on every screen and overlay.
- Confirm opens on Cancel; every dialog returns focus to its opener when one
  exists.
- The ring's measured contrast is reported per control kind and preset; nothing
  of arama's is under 3:1.
- The walk in §6.8 completes without a pointer.
- All gates green.

## 10. Prohibited shortcuts

- Wrapping an iced `checkbox`, `slider` or `pick_list` in a control. That
  publishes two messages per action (snora's own warning).
- Overriding snora's ring colour or width to fix the rail. That is snora's.
- Making a disabled control a stop.
- A key that does something different from the click.
- Merging into `main`. Deliver commits on a branch rebased onto `origin/main`
  (review 192).
- Running any arama binary except through `cargo test`, `cargo run` or
  `scripts/gui-session.sh`.

## 11. Module boundaries

UI crates' views, their state and updates where a widget needs state, and
`app/src/core/update` for focus tasks. No engine, cache or AI crate.

## 12. Compatibility constraints

Rust 1.94, snora 0.65, iced 0.14. Linux observed. macOS and Windows build and
run their native smoke in CI.

## 13. Security constraints

The confirm dialogs guard destructive actions. Starting on Cancel is the
security property here: no key press made before the dialog appeared can
confirm it.

## 14. Known risks

- **Look changes:** snora's checkbox, radio and slider are drawn differently from
  iced's. Show each changed screen before and after (one capture each: setup,
  settings general, a confirm, media focus).
- **Target sizes:** snora controls have a 24 px minimum target. Watch the footer
  and the header in Japanese at text size 1.5×, where T074 already fought for
  space. The matrix catches overflow.
- **A `select` inside a dialog:** Escape should close the select first, then the
  dialog. Observe it.

## 15. Required evidence

### 15.1 Evidence plan

| Question | Level | Evidence |
|---|---|---|
| Does every screen have its stops? | headless | the committed inventory test |
| Does focus start and return right? | unit | the per-dialog focus tests |
| Is the ring visible everywhere arama controls it? | numeric | §6.7's table |
| Does it work without a pointer? | real UI | §6.8's walk; captures: the first three Tabs, the confirm on Cancel, settings with a radio group focused |
| What changed in the look? | real UI | 4 before/after captures (§14) |
| Anything regressed? | machine | the gates, the matrix |

### 15.2 Deliverables

Commits on a branch rebased onto `origin/main`, and the rows above, observed.

## 16. Review-request format

A package under `.git-exclude/review-request/` with a `## Entry point` section:
the branch, the commits, the `git show` commands, and full repo-relative paths,
one per line, to the evidence files. Report observed output; a check not run is
recorded as not run. You cannot push: report the commits, and the architect
pushes.

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
