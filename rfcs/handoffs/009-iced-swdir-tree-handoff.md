# Handoff — RFC 009: Replace custom DirTree with iced-swdir-tree

**RFC.** [`rfcs/done/009-iced-swdir-tree.md`](../done/009-iced-swdir-tree.md)
**Shipped in.** v0.30.0

---

## 1. Implementation Handoff

### Goal
Delete arama's ~455-line custom `DirTree` widget and replace it with the
published `iced-swdir-tree` crate (v0.9.0, same author), gaining async
non-blocking directory scanning and 174 library tests.

### Why it's low-risk
The crate uses the exact dependency versions already in the workspace
(`iced 0.14`, `swdir 0.11`, `lucide-icons 1`, all optional) — **no new
transitive dependencies**.

### Key behaviour changes
- **Async scanning.** Expanding a directory issues an async `iced::Task`;
  the UI no longer blocks on a synchronous `swdir` walk.
- **`ensure_expanded` removed.** The crate natively shows all children on
  first expand, so the v0.29.0 workaround is deleted.
- **`Aside::new` loses `include_file` / `include_hidden`** (always `false,
  false`); `DirectoryFilter::FoldersOnly` encodes both.
- **`Aside` is no longer `Clone`** (`DirectoryTree` holds an executor handle;
  the derive was unused).

### Event routing (the important part)
`DirectoryTree::update` returns `Task<DirectoryTreeEvent>`. `Aside::update`
routes **all** variants back into the tree, and additionally emits the
app-visible `Event::DirSelect(path)` only on `Selected(path, true, _)`.
While `processing` is true, only `Loaded` / `Drag` variants are forwarded
(so in-flight scans complete); `Toggled` / `Selected` are dropped to block
navigation during indexing.

---

## 2. Task Breakdown / PR Plan

Single PR is acceptable (cohesive swap), but if splitting:

### PR 1 — Dependency + deletion
1. Add `iced-swdir-tree = { version = "0.9", features = ["icons"] }` to the
   workspace and `crates/ui/layout/Cargo.toml`.
2. Delete `crates/ui/widgets/src/dir_tree/` (8 files) and the
   `pub mod dir_tree` line in `crates/ui/widgets/src/lib.rs`.

### PR 2 — Aside rewrite + app wiring
3. Rewrite the four `aside/` files:
   - `aside.rs` — `DirectoryTree` field, simplified `new`, drop `Clone`,
     `update_dir_tree` rebuilds with `DirectoryFilter::FoldersOnly`.
   - `message.rs` — `Internal::TreeEvent(DirectoryTreeEvent)`.
   - `update.rs` — async routing + processing gate + `DirSelect` emission.
   - `view.rs` — `tree.view(|e| …TreeEvent(e))`.
4. `app/src/core.rs` — `Aside::new(PathBuf, processing)` (drop the two bool
   params). The app's `AsideMessage` handler needs no change (the
   `DirSelect` arm is unchanged).

---

## 3. Acceptance / QA Checklist

### Automated
- [ ] `cargo check --workspace` — zero errors, zero warnings.
- [ ] `cargo test -p arama-cache -p arama-i18n` — all pass.
- [ ] `Cargo.lock` gains `iced-swdir-tree` but **no** other new crates.

### Manual — directory tree
- [ ] The tree renders the root directory's folders on launch.
- [ ] Clicking a collapsed folder expands it and shows **all** immediate
      subfolders (not just one) — the bug `ensure_expanded` patched is gone.
- [ ] Expanding a large directory does **not** freeze the UI (async scan).
- [ ] Selecting a directory drives the gallery to that directory
      (`DirSelect` reaches the app).
- [ ] Navigating via the header path input rebuilds the tree at the new root.

### Manual — processing gate
- [ ] During an active caching/indexing run, clicking folders in the tree
      does not change the selection or start new scans (navigation blocked).
- [ ] A scan already in flight when indexing starts still completes
      (`Loaded` events not dropped) — the tree does not get stuck mid-expand.

### Regression
- [ ] Folder-only filtering: files do not appear in the tree.
- [ ] Hidden directories are not shown.
