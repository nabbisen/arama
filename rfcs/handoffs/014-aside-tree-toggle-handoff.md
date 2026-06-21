# RFC 014 Handoff — Explorer aside tree toggle

Companion to [RFC 014](../done/014-aside-tree-toggle.md). Shipped in **v0.36.1**.

## 1. Implementation Handoff

**Goal.** Replace the always-on aside panel (which caused scroll/width problems)
with a toggle-to-open, auto-close-on-select directory tree pane.

**Changed files:**

| File | Change |
|---|---|
| `app/src/core.rs` | Added `aside_open: bool` to `App` struct (init `false`) |
| `app/src/core/message.rs` | Added `ToggleAside` variant (ui housekeeping group) |
| `app/src/core/update.rs` | Routed `ToggleAside` to `handle_toggle_aside` |
| `app/src/core/update/ui.rs` | Added `handle_toggle_aside` (flips `aside_open`) |
| `app/src/core/update/component.rs` | Set `aside_open = false` before `on_dir_changed` in `handle_aside_message` |
| `app/src/core/view.rs` | Toggle button + conditional `aside` in Explorer tiling row |
| `crates/ui/layout/src/aside/view.rs` | Simplified to `FillPortion(1)` / `Fill` column |
| `crates/i18n/src/en.rs` | Added `aside.toggle.open` / `aside.toggle.close` keys |
| `crates/i18n/src/ja.rs` | Same keys in Japanese |

**UX flow:**
1. User lands on Explorer → gallery has full width; toggle shows `icon_panel_left_open`.
2. User clicks toggle → `aside_open = true`; tree pane appears at `FillPortion(1)`;
   gallery takes remaining space; toggle shows `icon_panel_left_close`.
3. User clicks a directory in the tree → `aside_open = false`; pane closes;
   gallery updates with the new directory's images.
4. User clicks toggle again (without selecting) → `aside_open = false`; pane closes;
   current directory unchanged.

**Key pitfalls:**
- `aside_open` is owned by `App`, not by `Aside` — keep it there.
- `Aside::view()` must not add an outer scrollable; `DirectoryTree::view()` already
  provides vertical scroll internally.

## 2. Acceptance / QA Checklist

- [ ] `cargo check --workspace` — clean.
- [ ] `cargo test -p arama-cache -p arama-i18n -p arama-env` — all pass.
- [ ] `cargo fmt --check` — clean.
- [ ] Toggle button visible left of header path input on Explorer page.
- [ ] Icon is `panel_left_open` when closed, `panel_left_close` when open.
- [ ] Button style is `ghost` when closed, `primary` when open.
- [ ] Selecting a directory closes the pane and updates the gallery.
- [ ] Closing via toggle (no selection) leaves current directory unchanged.
- [ ] No ELOC file exceeds 300.
