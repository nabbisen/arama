# RFC 009 — Replace custom DirTree with iced-swdir-tree

**Status.** Implemented (v0.30.0)
**Tracks.** Replacing the 455-line custom `DirTree` widget in
`crates/ui/widgets/src/dir_tree/` with the `iced-swdir-tree` crate
(v0.9.0, same author, zero new transitive dependencies).
**Touches.** `crates/ui/widgets/` (dir_tree deleted),
`crates/ui/layout/Cargo.toml` (new dep),
`crates/ui/layout/src/aside/` (full rewrite),
`app/src/core.rs` and `app/src/core/update.rs` (Aside construction
and message mapping).

## Summary

`iced-swdir-tree` is a batteries-included directory-tree iced widget
that covers everything arama's custom `DirTree` does, and more. It
uses exactly the same dependency versions already in the workspace
(`iced = "0.14"`, `swdir = "0.11"`, `lucide-icons = "1"`, all
optional). Replacing the custom widget removes ~455 lines of
untested code and gains 174 tests from the library, async
non-blocking directory scanning, and the correct "show all children
on first expand" behavior without the `ensure_expanded` workaround
added in v0.29.0.

## API mapping

| Custom DirTree | iced-swdir-tree |
|---|---|
| `DirTree::new(path, false, false, processing)` | `DirectoryTree::new(path).with_filter(DirectoryFilter::FoldersOnly)` |
| `DirTree::update_selected_path(path)` | Rebuild `DirectoryTree::new(path).with_filter(...)` |
| `DirTree::set_processing(bool)` | Stored on `Aside`; events gated in `Aside::update` |
| `dir_tree::message::Event::DirClick(path)` | `DirectoryTreeEvent::Selected(path, true, _)` |
| Internal expand/collapse messages | `DirectoryTreeEvent::Toggled` / `Loaded` (route back to `tree.update`) |

Key difference: `DirectoryTree::update` returns `Task<DirectoryTreeEvent>` —
scan tasks are async. `Aside::update` routes all variants back to
the tree; only `Selected(path, true, _)` also emits the app-visible
`Event::DirSelect`.

## Behaviour changes

- **Directory scans are non-blocking.** Clicking to expand a folder
  issues an async scan task; the UI does not block. Under the custom
  DirTree, swdir walked synchronously on the UI thread.
- **No `include_file` / `include_hidden` parameters on `Aside::new`.**
  These were always called with `false, false`; `DirectoryFilter::FoldersOnly`
  encodes both.
- **`Aside` is no longer `Clone`.** `DirectoryTree` holds an
  `Arc<dyn ScanExecutor>` which is not `Clone`. The custom `DirTree`
  derived `Clone` but it was never used in practice.
- **`ensure_expanded` removed.** `iced-swdir-tree` natively shows all
  children on first expand via its async scan-on-open mechanism.

## Processing flag

During an active indexing run (`processing = true`) the tree must not
forward user navigation events (expanding folders, selecting a new
directory). `Aside::update` filters: only `Loaded` and `Drag` variants
are forwarded to `tree.update` while processing is true; `Toggled`
and `Selected` are dropped, keeping the tree in a consistent state
(in-flight scans still complete).

## Touches in detail

| File / module | Change |
|---|---|
| `crates/ui/widgets/src/dir_tree/` | **Deleted** (all 8 files) |
| `crates/ui/widgets/src/lib.rs` | Remove `pub mod dir_tree` |
| `crates/ui/widgets/Cargo.toml` | Remove swdir dep (moved to layout) |
| `crates/ui/layout/Cargo.toml` | Add `iced-swdir-tree = { version = "0.9", features = ["icons"] }` |
| `crates/ui/layout/src/aside.rs` | `DirectoryTree` field; simplified `new`; drop `Clone` |
| `crates/ui/layout/src/aside/message.rs` | `Internal::TreeEvent(DirectoryTreeEvent)` replaces `DirTreeMessage` |
| `crates/ui/layout/src/aside/update.rs` | Async routing; processing gate; `DirSelect` on `Selected` |
| `crates/ui/layout/src/aside/view.rs` | `tree.view(...)` with `icons` feature |
| `app/src/core.rs` | `Aside::new` drops `include_file` / `include_hidden` params |
| `app/src/core/update.rs` | `Aside::update_selected_path` call sites unchanged |

## Open questions

None.
