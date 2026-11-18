> **AUTHORIZED 2026-11-18 — owner:** *"Authorized."* Written 2026-11-18 by the architect, as item 6 of the
> 0.44.0 plan (`.git-exclude/schedule/README.md`, owner: *"Authorized."*).
> **[RFC 048](../proposed/048-library-scale.md): only phase 1 is accepted.** This task
> implements only that phase (§0, §2, §4). The owner's *"Authorized."* answered
> the request to accept that phase and authorize this task together; phase 2 stays
> Proposed.
>
> **Independent of T085–T088.**

# T089 Handoff — files in a stable order, and no file-system calls while drawing

## 1. Purpose

Two small fixes a user notices, from RFC 048's phase 1:

- the gallery lists the files in each folder **in a stable, predictable order**,
  where today the order is arbitrary;
- **no `view()` touches the file system**. Today four do, on every redraw.

And one rename recorded since RFC 047: `Gallery::embedding_cached()` says
something it does not compute.

## 2. Background

- **Order (RFC 048 §2, audit A6).** `crates/ui/main/src/core/views/gallery.rs:15`:

  ```rust
  dir_path_thumbnail_path_map: BTreeMap<PathBuf, FastHashMap<String, String>>,
  ```

  Folders are ordered; **the files inside each folder are in hash order**:
  stable within a run, arbitrary between runs, meaningless to a user. The map
  is filled at `app/src/core/update/cache.rs:197`. `FastHashMap` is imported
  from `iced::wgpu::naga` (`gallery.rs:3`), the GPU shader compiler's hash map,
  an accidental dependency for a UI type.
- **I/O in `view()` (RFC 048 §4, audit C3).** Re-located on `main` 2026-11-18:

  | Site | Call |
  |---|---|
  | `crates/ui/main/src/core/views/setup/view.rs:85` | `DiskSpace::new` (`statvfs` / `GetDiskFreeSpaceEx`) |
  | `crates/ui/main/src/core/components/setup/downloader/view.rs:131` | `DiskSpace::new` |
  | `crates/ui/widgets/src/dialog/settings_dialog/tab/file_system_settings/view.rs:17` | `DiskSpace::new` |
  | `crates/ui/layout/src/footer/view.rs:35` | `canonicalize()` on the hovered path |

  The two setup sites run on the screen that animates the download progress, so
  they make a syscall on every frame of it. The footer canonicalizes on every
  frame of mouse movement over the gallery.
- **`embedding_cached()`** (`crates/ui/main/src/core/views/gallery.rs:31`)
  returns whether *any folder has two or more files*. It reads nothing about
  embeddings and takes `&mut self` for a read. Its one caller is
  `app/src/core/update/cache.rs:300`, which combines it with
  `embedding_completed` to enable *Similar pairs*. Review 134 already noted the
  mismatch.

## 3. Applicable RFC and requirements

[RFC 048](../proposed/048-library-scale.md) §0 (phase 1), §2, §4; §5 (what it
does not do: no sort control). RFC 047 §3.4 (the button's gate). The handoffs
README rules.

## 4. Change scope

- `crates/ui/main/src/core/views/gallery.rs`, `crates/ui/main/src/core/views/gallery/view.rs`
- `app/src/core/update/cache.rs` (the map's producer; the renamed call)
- `crates/ui/main/src/core/views/setup.rs`, `crates/ui/main/src/core/views/setup/view.rs`
- `crates/ui/main/src/core/components/setup/downloader/` (state and view)
- `crates/ui/widgets/src/dialog/settings_dialog/tab/file_system_settings/` (state and view)
- `crates/ui/layout/src/footer/` (state and view), and the message that sets the
  hovered path
- the tests next to each
- `CHANGELOG.md` `[Unreleased]`

## 5. Explicit non-change scope

- **No sort control, and no sort other than §6.1's.** Sorting by date, size or
  "natural" number order is product direction (RFC 048 §5).
- Gallery virtualisation and every other RFC 048 phase 2 item.
- The Similar pairs gate's logic: only the name changes (§6.3).
- `naga::FastHashMap` in `crates/ai` (`file_embedding_map.rs`): not part of
  the gallery's order. Note it for later; do not change it here.

## 6. Required implementation

1. **Order.** The inner map becomes a `BTreeMap<String, String>`, so files sort
   by their path string, which within one folder means by file name. That is
   **code-point order**: `B.jpg` before `a.jpg`, and `IMG_10` before `IMG_2`.
   That is what RFC 048 §2 asks for (stable and predictable), and the
   CHANGELOG says so plainly (§8). Change every producer and consumer of the
   type, and drop the `iced::wgpu::naga` import.
2. **Pure views.** For each of §2's four sites, the value moves into state, and
   the message that can change it refreshes it:
   - **setup, both sites:** when the screen opens, and on each download progress
     or completion message;
   - **settings, file system tab:** when the tab opens, and after a cache delete
     completes;
   - **footer:** canonicalize once in the update that handles the hover
     (`ImageCellEnter` or its successor), and store the string to show.

   Then **search every `view()` in `app/src` and `crates/ui`** for any other
   file-system call: `DiskSpace`, `canonicalize`, `metadata`, `exists`,
   `is_dir`, `is_file`, `read_dir`, `std::fs`. Report each hit, and either fix it
   or say why it is not I/O. `image::Handle::from_path` only records a path, and
   the renderer loads it later. Confirm this in iced's source rather than
   assuming it.
3. **The rename.** `Gallery::embedding_cached(&mut self)` becomes a `&self`
   method named for what it computes, for example
   `has_a_folder_with_two_files()`. The header's `set_embedding_cached` /
   `embedding_cached` field (`crates/ui/layout/src/header.rs`) means something
   else: "the Similar pairs button is enabled". Rename it to say that if it reads
   wrongly next to the new name, and say what you chose.

## 7. Required tests

- **Order:** a gallery built from files inserted in a shuffled order renders
  them sorted, on every run (Tier 1; assert the order of the rendered paths, not
  only the map's).
- **Pure views:** each moved value is set by the message named in §6.2 (Tier 1:
  send the message, read the state). And the reverse: a `view()` call with no
  message in between performs no new disk query. Use a counting test seam if you
  need one, kept `cfg(test)`.
- The existing Similar pairs gate tests still pass, unchanged except for names.
- Gates, all `--locked`, tests isolated; the layout matrix (RFC 053) unchanged
  except where the order of files is visible. Explain each such change.

## 8. Required documentation updates

`CHANGELOG.md` `[Unreleased]`:
- **Changed:** the files in each folder are listed in a fixed order, by name
  (capital letters before small, and `10` before `2`); before, the order could
  change between runs.
- **Fixed:** the setup screen, the file-system settings and the footer no longer
  query the disk on every redraw.

## 9. Acceptance criteria

- The gallery's file order is deterministic and sorted, proven by a test.
- No `view()` performs file-system I/O; the search in §6.2 is reported in full.
- `embedding_cached()` is gone under that name.
- All gates green; matrix changes explained.

## 10. Prohibited shortcuts

- Sorting inside `view()` on every frame instead of changing the type.
- Caching a disk-space value without a message that refreshes it, so that it
  goes stale.
- A "natural" or locale-aware sort (§5).
- Running any arama binary except through `cargo test`, `cargo run` or
  `scripts/gui-session.sh`.

## 11. Module boundaries

UI crates' state and views, and the producer in `app/src/core/update/cache.rs`.
No engine, cache or AI crate changes.

## 12. Compatibility constraints

Code-point order is identical on every system. `DiskSpace` already has a
per-system implementation; only where it is called changes.

## 13. Security constraints

None.

## 14. Known risks

- **A stale disk-space figure** during a long download, if the refresh messages
  are too rare. Name the message that refreshes it, and show that it runs during
  a download.
- **The footer** shows the canonical path, which can differ from the listed one
  (symlinks). Keep exactly today's output; only move where it is computed.

## 15. Required evidence

### 15.1 Evidence plan

| Question | Level | Evidence |
|---|---|---|
| Is the order fixed and sorted? | unit | the order test |
| Do views stay pure? | unit + source | the message tests, the no-query test, the §6.2 search output |
| Anything regressed? | machine + headless | the gates, the matrix |

No real-UI capture needed. The order shows in the matrix if a populated state
exists; if not, the unit test is the evidence.

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
