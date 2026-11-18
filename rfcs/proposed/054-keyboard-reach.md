# RFC 054: Keyboard reach — everything a pointer does, the keyboard does

**Status:** **Accepted by the project owner, 2026-11-18** (*"Accepted."*). Written 2026-11-18 by the architect, as item 1 of the
0.44.0 plan (owner: *"Authorized."*). Acceptance of this RFC is not authorization
of T085–T087; each has its own banner.
**Raised by:** the snora 0.65.0 notice (`.git-exclude/upstream/snora/receive/0.65.0/`)
and the architect's check of arama against it, 2026-11-18
**Relates to:** [RFC 044](../done/044-keyboard-operability.md) (amends its §2.2
and answers its §3.2 and §3.3), [RFC 048](./048-library-scale.md) (the gallery's
scale), [RFC 052](../done/052-text-size-and-reach.md)

## Summary

RFC 044 set the goal: WCAG 2.1 SC 2.1.1, *all functionality operable through a
keyboard interface*. It shipped what snora could then support: Escape closes a
dialog, and F6 moves a ring between regions. **The goal is still not met**, and
in one place the screen now says more than arama does:

- **Tab does nothing.** iced 0.14 binds no Tab key, and arama never routes it
  (`app/src/core/update/keyboard.rs` handles F6 only). *From the code; T085
  observes it in a window.*
- **F6 moves a ring, not focus.** `snora::focus::next_zone` could choose a
  region but not move focus into it (snora's 0.60 → 0.61 guide). The footer says
  *"F6 switches panels"*; after F6, Enter and Space act on nothing.
- **Almost nothing can take focus.** The 5 `text_input`s and snora's own
  controls (the rail, dialog chrome) can. arama's **36 `button`s, 2 `checkbox`es,
  2 `slider`s, 1 `pick_list` and 10 `mouse_area` click targets** cannot. The click
  targets include every gallery thumbnail: **a keyboard-only user cannot open a
  picture.**

**snora 0.65 now provides every mechanism this needs** (§2), and the upgrade is
T085. This RFC decides how arama uses them: which things take focus, in what
order, where focus starts and returns, and — the one real design question — how
a keyboard user moves through a gallery of thousands of files without thousands
of Tab stops.

## 1. Principles

1. **Everything a pointer does, a key does** (SC 2.1.1). Not "most things".
2. **The screen never promises a key that does nothing.** The F6 hint is true
   after this RFC, or it goes.
3. **Tab crosses the window in a bounded number of steps, whatever the library
   size.** A collection that grows with the user's files (the gallery, the
   similar-pairs list, the media focus strip, the folder tree) is **one** Tab
   stop, with arrow keys inside it. This is the WAI-ARIA grid/listbox pattern,
   and snora's own `radio_group` already works this way.
4. **Use snora's mechanisms; build none in parallel.** Where snora lacks one,
   arama builds the smallest piece, and says so to snora (owner's outbound
   decision).
5. **A click does not show a ring** (snora RFC-121, adopted as is): a pointer
   user sees nothing new, and a later Tab carries on from what they clicked.

## 2. What snora 0.65 provides (verified in its source, 2026-11-18)

| Need | snora 0.65 | Notes |
|---|---|---|
| Tab / Shift+Tab | `snora::keyboard::traverse(key, modifiers)` | Plain Tab only. Resumes after the last clicked control. Does not wrap: past the last stop, focus leaves, and the next Tab starts again from the top. |
| F6 moves real focus | `snora::keyboard::focus_zone(cycle)` with `cycle_zones` | To the first control of the next region (header, side bar, body, footer). The app keeps **no** zone state. |
| A focusable button | `snora::control(content)` | Takes `.style(fn(&Theme, button::Status) -> button::Style)`, so arama's `arama_theme::{primary, ghost, secondary, danger}` carry over unchanged. `.id()`. |
| Form controls | `snora::design::{checkbox, toggler, radio_group, select, slider}` | Each one Tab stop. A radio group is one stop for the group; ↑ ↓ ← → move and select. |
| Containment in a dialog | built into `snora::render` | With a dialog, sheet or menu open, Tab, Enter and shortcuts reach only it and the toasts. RFC 044 §3.3's open question is closed by snora. |
| Menus by keyboard | built into `snora::render` | ↓ ↑ Home End; Enter presses; Escape closes. |
| Where focus starts | `.id("…")` + `iced::widget::operation::focus("…")` | Returned from the update that opens the dialog. |

**Not provided:** a *roving* group for content of the application's own (a grid
of thumbnails). §3.5 covers that.

## 3. Decisions

### 3.1 Tab and F6 (T085)

- `handle_key_pressed` routes `traverse` first, then `cycle_zones` → `focus_zone`.
  Under an open dialog both work as snora defines them.
- **Remove** the `focus_zone` field, `focus_visible`, `zone_presence()` and the
  region ring (`arama_theme::focus_ring_style` on the side bar, body and
  footer). With real focus, the focused control draws its own ring. A second ring
  around a region would show two "focused" places at once.
- **Keep the footer hint** *"F6 switches panels"*: after T085 it is true. T085
  checks this in a window: F6 then Enter acts on the region's first control.
- **Tab order is the widget tree's order.** arama sets no order of its own;
  snora and iced follow the tree. T086 records the order on each screen and
  checks that it reads in the order things are drawn. Where it does not, the fix
  is the layout, not an order table.

### 3.2 Every pointer action is a control (T086)

| Today | Becomes | Why this one |
|---|---|---|
| 36 `iced::widget::button` | `snora::control`, same style function | One Tab stop each, same look. |
| `checkbox` ×2 (include images / videos) | `snora::design::checkbox` | |
| `slider` ×2 (similarity threshold, thumbnail size) | `snora::design::slider` | Arrows, Page Up/Down, Home/End. |
| `pick_list` ×1 (media focus: strategy) | `snora::design::select` | |
| **Language, Theme, Text size**: one-of-N button rows | `snora::design::radio_group`, one per setting | They *are* radio groups: one choice of N. One Tab stop per setting instead of 2 + 4 + 3. |
| About: repository link (`mouse_area` on text) | `snora::control`, styled as a link | |

**Look:** each swap keeps the current look where snora allows it. Where snora's
control draws differently (a checkbox's box, a radio's marks), snora's look
wins, because it carries the focus ring and the 3:1 guarantees. T086 shows each
changed screen in the layout matrix, before and after.

### 3.3 Where focus starts, and where it returns (T086)

Every dialog sets focus when it opens, so Enter never acts on something hidden
behind it.

> **Corrected 2026-11-18 (review 189).** This table used to list Settings.
> Settings is a page (`NavPage::Settings`), not a dialog: it is reached from the
> rail, Escape has nothing to close, and focus stays on the rail item that
> opened it. The dev team found this in T085.

| Dialog | Focus starts on | Reason |
|---|---|---|
| Confirm (cache delete) | **Cancel** | Destructive: Enter on open must not delete. |
| Media focus | the current item of the similar strip (§3.5); Close if the strip is empty | |
| Similar pairs | the first pair (§3.5); Close while scanning or empty | |

**On close, focus returns to the control that opened the dialog** (SC 2.4.3,
focus order). The opening controls get ids; the update that closes the dialog
returns `operation::focus(opener)`. When the opener no longer exists (the gallery
reloaded), focus goes nowhere, and the next Tab starts from the top (snora's
rule for a missing id).

### 3.4 Focus is visible (T086)

snora draws the ring on its controls and holds it to 3:1 on the design path.
**arama gives most controls its own `.style()`**, so T085 checks that the ring
is still drawn on a control with a host style (Phase 0, Q4). T086 then measures
it against its background on all four presets, as T064 measured the rail's
active fill (`crates/theme/src/lib.rs`).

### 3.5 Collections: one Tab stop, arrow keys inside (T087)

The gallery, the similar-pairs list, the media focus strip and the folder tree
grow with the library. Per principle 3 each is **one** Tab stop.

**Decision: arama renders only the collection's *current* item as a
`snora::control`; every other item stays a plain click target.** The current
item is the one Tab lands on. arama answers the arrow keys while that control
has focus: it moves the current index (← → by one, ↑ ↓ by a row of the grid,
Home/End to the ends), and returns `operation::focus(id)` for the new item, so
the ring moves with it. Enter opens the item, the same as a double click today.
A click on any item makes it current, so a later Tab or arrow carries on from
there (principle 5).

- **Why this and not one control per item:** it is a roving tab stop in the
  simplest form. Tab cost and focus-walk cost are constant, not proportional to
  the library: whether a walk over thousands of controls is cheap enough is an
  RFC 048 question, and this design never asks it.
- **What it needs:** to tell that focus is on the current item (Phase 0, Q1).
  An arrow key reaches `keyboard::listen()` only when nothing captured it.
  `text_input`, snora's slider, radio group and menus all capture arrows, so a
  free arrow means focus is on a plain control, or nowhere. That is not enough:
  an arrow pressed on the *Similar pairs* button must not move the gallery.
  **Answered by T085 (review 189): yes.** `focusable::find_focused()` returns a
  `snora::control`'s own `.id()`, at about 44 µs per call over 1,000 widgets.
  In production it needs either iced's `advanced` feature or a direct
  `iced_runtime` dependency (both already in the lockfile; the probe used
  `iced_test`'s re-exports, which only tests have). It runs as a `Task` that
  reports the focused id as a message, so an arrow becomes key → task → message
  → move. T087 chooses between the two routes.
- **If Q1 cannot be answered within arama:** ask snora for a roving-group
  primitive, the `radio_group` mechanism opened to the application's own
  content. Ship the gallery as one Tab stop where Enter opens the current item,
  without arrows, and say so in the release notes. **Not** one Tab stop per
  thumbnail.

**The folder tree** (`iced-swdir-tree` 0.9, the owner's crate) already maps
arrows, Home/End, Enter and ← → to tree events
(`DirectoryTree::handle_key`). It is *focus-neutral* by design: the application
decides when the tree "has focus". arama does not call it today. It needs the
same thing as the gallery: one focus target that stands for the tree (Phase 0,
Q3). If that is not clean from arama's side, the place to fix it is the tree
crate itself, the same route `pt2safetensors` took in 0.43.0. **Choosing a folder
by keyboard does not wait for the tree:** the header's folder field (type a path,
Enter) and *Folder…* (the system's own dialog) are both reachable once T086 has
made them controls.

### 3.6 The context menu by keyboard (T087)

The gallery's right-click menu has two actions: *Open with default app* and
*Show in file manager*.

- **Required:** the **media focus dialog** gets both actions as controls. A
  keyboard user reaches them with Enter on a thumbnail, then Tab. Pointer users
  gain them too.
- **Desirable:** Shift+F10 and the Menu key open the context menu for the current
  item, at the item. This needs Q1, and the item's position (Q6). The menu's own
  keys are snora's.

## 4. Phase 0 — questions T085 answers before T086 and T087 start

| Q | Question | Decides | Answered by |
|---|---|---|---|
| Q1 | Can arama tell which control holds focus (iced's `advanced` feature, `focusable::find_focused`, or a snora API)? At what cost per key? | §3.5, §3.6 | a probe in a scratch worktree; report the API and a timing |
| Q2 | Does the region ring removal leave any region without a visible focus target after F6 (the footer before T086 has only an iced slider)? | §3.1 order of landing | observe in a window |
| Q3 | Can the folder tree get one focus target from arama's side, without wrapping its rows in a control (a click on a row must stay a row click)? | §3.5 tree | probe; if not, a note for the tree crate |
| Q4 | Does snora draw its focus ring on a `snora::control` given a host `.style()`? | §3.4 | observe on two presets |
| Q5 | With snora 0.65, are any arama screens drawn differently (0.62's 17 state colours, 0.64's progress bar)? | T085 evidence | the layout matrix, before/after |
| Q6 | Can arama know a gallery item's on-screen position without a pointer event? | §3.6 desirable part | probe |

A "no" to Q1 changes §3.5 to its fallback. It does not stop the release.

## 5. Tasks

| Task | Covers | Lands |
|---|---|---|
| T085 | snora 0.57 → 0.65; §3.1; Phase 0 | first |
| T086 | §3.2, §3.3, §3.4 | after T085 |
| T087 | §3.5, §3.6, a populated gallery in the layout matrix | after T086 |

## 6. Acceptance

**One keyboard-only walk, with no pointer event**, through
`scripts/gui-session.sh` and `xdotool key` (which also bypasses the open
`gui-session.sh` problem that clicks do not reach arama). Fresh isolated profile:

1. First-run setup: download (or skip) by keyboard.
2. Choose a fixture folder with the folder field.
3. Let it index. Move through the gallery with arrows; open an image with Enter.
4. In the media focus dialog: change the strategy, use one of the two file
   actions, close; focus is back on the thumbnail.
5. Open Similar pairs; move through it; open a pair; close.
6. Settings (a page, reached from the rail): change the language, the theme and
   the text size; toggle *include videos*; move the similarity slider; return to
   the explorer from the rail.
7. Cache page: filter; delete. The confirm dialog opens on **Cancel**; Enter
   cancels; then Tab to the delete action and confirm.
8. F6 through every region, forward and back.

At each step: the ring is visible, and nothing outside an open dialog reacts.
Plus Tier 1 tests for each pure decision (the arrow mapping, the focus target on
open and close), and §3.4's contrast measurement.

## 7. Non-goals

- **Screen readers.** Keyboard operation is not exposure to assistive
  technology. This RFC makes arama usable without a pointer. It does not make
  arama readable by a screen reader, which needs an accessibility tree that
  neither arama nor snora provides today. The release notes must not claim it.
- **New global shortcuts** (Ctrl+O and similar). A separate question.
- **Right-to-left.** arama ships no RTL locale; snora's direction stays `Ltr`.
- **Gallery virtualisation** (RFC 048 phase 2). §3.5 does not depend on it, and
  does not block it.

## 8. For the release notes (draft)

> **arama can now be used without a mouse.** Tab and Shift+Tab move between
> controls, and F6 between panels. Arrow keys move through the gallery and lists;
> Enter opens. Every dialog starts on a safe choice, and closing it returns you
> to where you were. Settings' checkboxes, sliders and choices are now reachable
> by keyboard.
