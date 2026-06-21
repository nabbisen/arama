# RFC 014 — Explorer aside tree toggle

**Status.** Implemented (v0.36.1)
**Tracks.** v0.36.1 UX fix — replace the always-visible, scrollbar-heavy
aside panel with a toggle-to-open/auto-close directory tree pane.
**Touches.** `app/src/core.rs` (App struct), `app/src/core/message.rs`,
`app/src/core/update/ui.rs`, `app/src/core/view.rs`,
`crates/ui/layout/src/aside/view.rs`,
`crates/i18n/src/en.rs`, `crates/i18n/src/ja.rs`.
No new crates; no logic changes to the cache pipeline or gallery.

---

## Summary

The current aside panel is always visible in the Explorer tiling row.
Giving it a fixed pixel width clips deep paths; giving it `FillPortion`
plus `Direction::Both` scrollbars is visually confusing. The root problem
is that the tree is a transient navigation tool, not persistent content —
the user opens it, picks a folder, then returns focus to the gallery.

This RFC replaces the always-on panel with a **toggle button** in the
Explorer header area. Clicking opens the tree pane; selecting a directory
closes it automatically; a second click on the toggle closes it without
selecting. When closed, the gallery has the full width.

---

## Design

### State

One new boolean field on `App`:

```rust
// app/src/core.rs  — App struct
aside_open: bool,   // false by default; persisted only for the session
```

### Message

One new variant in the router (ui housekeeping group):

```rust
// app/src/core/message.rs
ToggleAside,
```

### Toggle button placement

The toggle sits at the left end of the Explorer header row — before the
directory input — using the lucide `icon_panel_left_open` /
`icon_panel_left_close` icons to indicate current state:

```
[ ▣ ] [ /path/to/dir _________________ ] [📁] [⚡]
  ↑ toggle    ↑ DirNav input                   ↑ SimilarPairs
```

The button is rendered inside `app/src/core/view.rs` (Explorer branch)
rather than inside `arama-ui-layout`'s Header component, to keep the
toggle's state (`aside_open`) owned by `App` where it belongs.

### View logic

```
NavPage::Explorer:
  if aside_open:
      row![ aside (FillPortion 1) | gallery (Fill) ]
  else:
      gallery alone (full width, Fill)
```

`Aside::view()` is called only when `aside_open` is true, so the
`DirectoryTree` widget is not in the element tree when the pane is
closed — no layout cost, no scroll artifacts.

### Auto-close on selection

`handle_aside_message` already handles `Event::DirSelect`. After calling
`on_dir_changed`, set `self.aside_open = false`.

### i18n keys (new)

| Key | EN | JA |
|---|---|---|
| `aside.toggle.open` | `"Open folder tree"` | `"フォルダーツリーを開く"` |
| `aside.toggle.close` | `"Close folder tree"` | `"フォルダーツリーを閉じる"` |

These are tooltip strings for the toggle button.

### Aside view cleanup

With no always-on visibility requirement, `Aside::view()` can drop the
outer `scrollable` wrapper and `FillPortion` and return to a clean
`Length::Fill` column — the toggle ensures the tree is only in the layout
when explicitly opened.

---

## Non-goals

- No persisting `aside_open` across app restarts (session-only state).
- No animation or slide transition.
- No change to `Aside` struct, `DirectoryTree` internals, or the cache
  pipeline.

---

## Task breakdown

1. `app/src/core.rs` — add `aside_open: bool` to `App` struct (default
   `false`).
2. `app/src/core/message.rs` — add `ToggleAside` variant.
3. `app/src/core/update/ui.rs` — add `handle_toggle_aside`; flip
   `aside_open`.
4. `app/src/core/update.rs` — route `Message::ToggleAside`.
5. `app/src/core/update/component.rs` — set `aside_open = false` after
   `DirSelect` in `handle_aside_message`.
6. `app/src/core/view.rs` — toggle button + conditional aside in tiling
   row.
7. `crates/ui/layout/src/aside/view.rs` — simplify back to `Fill` column.
8. `crates/i18n/src/en.rs` + `ja.rs` — add two i18n keys.
9. `cargo fmt` once; `cargo check --workspace`; tests.

## Acceptance / QA checklist

- [ ] `cargo check --workspace` clean.
- [ ] `cargo test -p arama-cache -p arama-i18n -p arama-env` — all pass.
- [ ] `cargo fmt --check` clean.
- [ ] Toggle button visible in Explorer header; correct icon per state.
- [ ] Opening the pane shows the directory tree; gallery shrinks to
      `FillPortion`.
- [ ] Selecting a directory closes the pane and updates the gallery.
- [ ] Closing via toggle (without selecting) leaves the current directory
      unchanged.
- [ ] No `.rs` file exceeds 300 ELOC.

## Implementation notes

- `aside_open` is initialised to `false`; gallery has full width on startup.
- Toggle button is rendered in `app/src/core/view.rs` (Explorer branch),
  keeping `aside_open` state owned by `App`.
- `row![toggle, header]` with `align_y(Alignment::Center)` keeps the toggle
  visually aligned with the DirNav input.
- Auto-close is a single `self.aside_open = false` before `on_dir_changed`
  in `handle_aside_message` — no extra message round-trip needed.
- `Aside::view()` returns a clean `FillPortion(1)` / `Fill` column;
  the double-scrollbar problem from the previous approach is eliminated.
