# RFC 013 — ELOC splits: `app/src/core/update.rs` and `crates/cache/tests/integration_tests.rs`

**Status.** Implemented (v0.36.0)
**Tracks.** Two files that exceed the 500 ELOC threshold identified during
the v0.35.0 baseline review and deferred from RFC 012.
**Touches.** `app/src/core/update.rs` (608 lines, 543 ELOC) and
`crates/cache/tests/integration_tests.rs` (774 lines, 615 ELOC). No
behaviour changes; no public API changes; no new logic.

---

## Summary

Both files are over the 500 ELOC "strongly recommended split" threshold.
This RFC splits each along its natural logical boundary: `update.rs` by
handler domain, and `integration_tests.rs` by cache namespace (image vs.
video) plus shared helpers. The result lowers every file in the project
to below 300 ELOC, making each file's responsibility immediately legible
from its name.

No logic is changed. The goal is purely structural — files that are
easier to navigate, review, and extend.

---

## (a) `app/src/core/update.rs` — split by handler domain

### Current structure

`update.rs` holds a single `impl App` block with one large `match
message` arm dispatch plus three private helpers:

```
impl App {
    pub fn update(&mut self, message: Message) -> Task<Message>  // the big match
    fn on_dir_changed(...)
    fn on_cache_page_request(...)
    fn run_finished_reload(...)
}

fn clear_dir_task(...)           // free async task constructor
fn dir_path_thumbnail_path_map(...)  // free recursive helper
```

The match arms naturally group into four domains:

| Domain | Arms |
|---|---|
| **Navigation** | `NavTo` |
| **Cache pipeline** | `CacheRequire`, `ThumbnailCacheFinished`, `EmbeddingCacheFinished` |
| **Component delegation** | `CachePageMessage`, `CacheClearFinished`, `SetupMessage`, `GalleryMessage`, `HeaderMessage`, `AsideMessage`, `FooterMessage`, `MediaFocusDialogMessage`, `SimilarPairsDialogMessage`, `SettingsDialogMessage`, `ContextMenuMessage` |
| **UI housekeeping** | `DialogClose`, `CloseMenus`, `ToastDismiss`, `ToastSweep`, `CursorMove` |

### Design

Rust 2018+ module style: `update.rs` coexists with an `update/`
subdirectory; no `mod.rs` required.

```
app/src/core/
  update.rs          ← pub fn update() entry; re-exports nothing new;
                       delegates to sub-handlers via private fn calls
  update/
    cache.rs         ← CacheRequire + ThumbnailCacheFinished +
                       EmbeddingCacheFinished handlers + helpers:
                       on_dir_changed, on_cache_page_request,
                       run_finished_reload, clear_dir_task,
                       dir_path_thumbnail_path_map
    component.rs     ← all component-delegation arms (11 arms)
    ui.rs            ← nav + UI housekeeping arms (6 arms)
```

`update.rs` becomes a thin router — the `match message` body consists
only of `self.on_*(…)` or `self.handle_*(…)` delegating calls, each
defined in the relevant sub-file. No logic moves between files; only its
location changes.

**Approximate ELOC after split:**

| File | Estimated ELOC |
|---|---|
| `update.rs` (router) | ~40 |
| `update/cache.rs` | ~230 |
| `update/component.rs` | ~180 |
| `update/ui.rs` | ~60 |

All under 300 ELOC; none approach 500.

---

## (b) `crates/cache/tests/integration_tests.rs` — split by namespace

### Current structure

The file covers two independent cache namespaces and one cross-cutting
concern, with shared test helpers at the top:

```
// shared helpers (TempFile, tmp_db, image_writer_with_db, upsert_image, ...)
// image tests (10 tests: upsert, lookup, invalidate, thumbnail, upsert_all, …)
// video tests (7 tests: upsert, coalesce, miss, invalidate, upsert_all, …)
// cross-namespace tests (5 tests: delete, list, onetime, readers, parallel, summarize)
```

### Design

Per guidelines: test code in `src/` → `tests.rs`; if that grows → `tests/`
subdirectory. The cache crate uses a `tests/` directory at crate root
already (`crates/cache/tests/integration_tests.rs`). The same sub-module
principle applies: split into files under a `tests/` folder.

Rust 2018+ module style: `integration_tests.rs` declares the submodules;
the files live in `tests/integration_tests/`.

```
crates/cache/tests/
  integration_tests.rs          ← mod declarations + shared imports only;
                                   TempFile helper stays here (used by all)
  integration_tests/
    helpers.rs                  ← shared fixture functions: tmp_db,
                                   image_writer_with_db, upsert_image,
                                   video_writer, files_in_dir
    image.rs                    ← 10 image-namespace tests
    video.rs                    ← 7 video-namespace tests
    cross.rs                    ← 5 cross-namespace / session / parallel tests
```

**Approximate ELOC after split:**

| File | Estimated ELOC |
|---|---|
| `integration_tests.rs` (entry) | ~20 |
| `integration_tests/helpers.rs` | ~60 |
| `integration_tests/image.rs` | ~200 |
| `integration_tests/video.rs` | ~130 |
| `integration_tests/cross.rs` | ~160 |

All under 300 ELOC.

---

## Non-goals

- No logic changes, refactors, or behaviour changes of any kind.
- No new tests added (test additions belong in a feature RFC).
- No changes to `crates/cache/src/` (only `tests/` is touched).
- No changes to any other file in the workspace.

---

## Task breakdown

1. Split `app/src/core/update.rs` → `update.rs` + `update/{cache,component,ui}.rs`.
2. Split `crates/cache/tests/integration_tests.rs` → `integration_tests.rs` +
   `integration_tests/{helpers,image,video,cross}.rs`.
3. `cargo fmt` once after both splits are complete.
4. `cargo check --workspace` and `cargo test --workspace` must pass clean.
5. Confirm no ELOC in any `.rs` file exceeds 300.

## Acceptance / QA checklist

- [ ] `cargo check --workspace` — clean.
- [ ] `cargo test --workspace` — all tests pass, count unchanged.
- [ ] `cargo fmt --check` — clean.
- [ ] Every `.rs` file in `app/` and `crates/cache/tests/` is under 300 ELOC.
- [ ] `git diff --stat` shows only file additions and deletions (no unrelated
      changes); diff of concatenated content is empty.

## Implementation notes

### update.rs split

The split landed cleanly. The router (`update.rs`) is 35 ELOC; the three
sub-files are all under 300 ELOC. One pre-existing bug was found and fixed:
`component.rs` had a stale import at the wrong `super` depth for
`ContextMenuState`; corrected to `use arama_ui_widgets::context_menu::ContextMenuState`.

### integration_tests.rs split — module resolution in tests/

The RFC's proposed layout (`integration_tests/helpers.rs`, etc.) was incorrect.
The Rust 2018+ "foo.rs + foo/ coexistence" rule applies only when the
declaring file's stem matches the subdirectory. `integration_tests.rs` is the
root; `mod image` therefore looks for `tests/image.rs` (sibling), not
`tests/integration_tests/image.rs`.

Final layout: `helpers.rs`, `image.rs`, `video.rs`, `cross.rs` are all
siblings of `integration_tests.rs` under `crates/cache/tests/`.
Each of `image.rs`, `video.rs`, and `cross.rs` is its own Cargo test
binary and includes `helpers.rs` via `#[path = "helpers.rs"] mod helpers`.
`autotests = false` and four explicit `[[test]]` entries were added to
`crates/cache/Cargo.toml` to prevent Cargo auto-discovering `helpers.rs`
as a standalone test binary.
