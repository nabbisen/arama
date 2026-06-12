# Changelog

All notable changes to arama are documented here.
Releases follow the archive naming `arama-vX.Y.Z.tar.gz`.

---

## [Unreleased]

### Planned

- Relative-time rendering ("2 days ago") for the Cache page table.

### Added

- **Smoke tests for `arama-i18n`** (`locale_round_trip`, `translation_and_fallback`).
  Cover locale switching, the current→English→raw-key fallback chain, and the
  `Locale` code/display-name accessors. The crate has zero heavyweight dependencies
  so the test binary is fast to build.
  `iced_test` was evaluated as a candidate for view-layer smoke tests but not
  adopted: its `Simulator` links the full iced rendering stack (wgpu, winit,
  wayland, tiny-skia) even for headless tests, making test builds prohibitively
  heavy with no proportionate benefit for a project whose testable logic lives
  outside the view layer.

### Dependency updates applied after migration analysis

API-level source analysis (diffing public symbols across registry
sources) confirmed both updates are drop-in for arama's usage.
Migration reports at `rfcs/notes/dep-migration-lucide-icons.md` and
`rfcs/notes/dep-migration-candle.md`.

- **`lucide-icons` 0.576.0 → 1.17.0.** The 20 removed icons are all
  brand/social-media icons (Twitter, GitHub, Figma, etc.); none are
  used in arama. The `iced` feature and all function signatures are
  unchanged. Workspace constraint updated to `"1"`.

- **`candle-core` / `candle-nn` / `candle-transformers` 0.9.2 → 0.10.2.**
  Zero items removed from any of the three crates. Every struct, trait,
  and function that arama-ai imports exists unchanged in 0.10.2. The
  two additions (`TokenizerFromGguf` in core, `remove_mean` in nn) are
  unrelated to arama's CLIP/wav2vec2 pipeline. Constraints in
  `arama-ai/Cargo.toml` updated to `"0.10"`.

---

## [0.30.0]

### Changed

- **Replace custom `DirTree` with `iced-swdir-tree` 0.9.0** (RFC 009).
  The 455-line custom directory-tree widget in `crates/ui/widgets/src/dir_tree/`
  is removed and replaced by the `iced-swdir-tree` crate (same author; uses
  the same `iced 0.14`, `swdir 0.11`, and `lucide-icons 1` versions already
  in the workspace — no new transitive dependencies).

  Behaviour changes:
  - **Async scanning.** Expanding a directory no longer blocks the UI thread;
    it issues an async `iced::Task` that merges the result back when complete.
  - **`ensure_expanded` removed.** `iced-swdir-tree` natively shows all
    children on first expand, making the workaround added in v0.29.0 unnecessary.
  - **`Aside::new` simplified.** The `include_file` and `include_hidden`
    parameters are removed; `DirectoryFilter::FoldersOnly` encodes both.
  - **`Aside` is no longer `Clone`.** `DirectoryTree` holds an executor
    handle; the derive was unused.

---

## [0.29.0]

### Added

- **Gallery filename filter** (RFC 008). A search row above the
  thumbnail grid lets users filter visible files by filename substring
  (case-insensitive). The filter row shows a text input, a clear (✕)
  button, and a `N of M` count while filtering. Only matching files are
  shown; directory group labels are preserved as long as they have at
  least one matching entry. The filter resets automatically when the
  selected directory changes.

### Fixed

- **AI pipeline debug output removed** (RFC 008). Development
  `println!` calls annotated `// todo: delete debugger` have been
  removed from `video_extractor.rs`, `clip_encoder.rs`, and
  `video_similarity_pipeline.rs`. Frame extraction errors are now
  reported via `eprintln!` (prefixed `arama:`) rather than stdout.

- **Error handling sweep** (RFC 008).
  - `Setup::default()` failure is caught and surfaced as a startup
    error toast rather than panicking; the app falls back to a
    `Setup::fallback()` state that skips the wizard.
  - `set_extension_allowlist()` failure in `dir_node()` degrades
    gracefully to an unfiltered directory walk instead of panicking.
  - The thumbnail cache writer construction inside the async indexing
    task uses an early `return vec![]` instead of `.expect()`.
  - `SimilarPairsDialog` now checks for a `None` directory node and
    shows an error toast ("Select a directory first.") rather than
    unwrapping unconditionally.
  - Stale `// todo` placeholder comments removed from
    `media_focus_dialog/view.rs` and `similar_media.rs`.

---

## [0.28.0]

### Changed

- **i18n Phase 2 sweep** (RFC 007). All remaining hardcoded English
  strings translated to use `t()`. Six views covered:

  **Setup wizard** — "Download" / "Skip" buttons, "Not enough space"
  message, all three component names (CLIP, wav2vec2, ffmpeg),
  download-state labels (Missing, Downloading, Ready, Error), and the
  disk-space display.

  **Focus dialog** — "Cache lookup strategy" label, "Close" button.

  **Similar-pairs dialog** — "No valid pairs." empty state.

  **Header folder-picker** — "Folder" button label.

  **Gallery** — "No file to render." empty state.

- **Panic removed.** `state_name()` in the setup downloader no longer
  panics on an unrecognised AI model config; it falls back to the CLIP
  label and logs via `eprintln!`. Typo "donwload" corrected throughout.

- **Code comment language.** Japanese comments in `gallery/view.rs`
  translated to English (project convention).

---

## [0.27.0]

### Added

- **Stop button on Cache page ⏳ row.** While a caching run is active
  the in-progress row shows a ◉ stop button next to the "⏳ caching…"
  indicator. Pressing it aborts the active task via the existing
  `task_handle` and reloads the table.

- **Multilingual GUI — i18n foundation** (RFC 006). A new zero-dependency
  `arama-i18n` workspace crate (`crates/i18n/`) exposes `t(key)`,
  `set_locale(Locale)`, and `current_locale()`. The active locale is
  stored in a global `AtomicU8` — lock-free, callable from any thread.
  Fallback chain: current locale → English → raw key string, so
  partially-translated locales degrade gracefully.

  **English and Japanese** locale tables ship for the Settings page
  (all four tabs), the Cache page, and the side-nav tooltips.
  `Settings::locale: Locale` (serde default `En`) is persisted across
  restarts. A language selector (EN / 日本語 buttons) in Settings →
  General changes the locale immediately with no restart required.

  Phase 2 (gallery, focus dialog, similar-pairs dialog, setup wizard)
  is tracked in Unreleased.

---

## [0.26.0]

### Added

- **Configurable similarity threshold** (RFC 005). A labeled slider
  (range 0.50–1.00, step 0.01) in Settings → General replaces the
  hardcoded 0.86 constant that was marked `// todo ui sliders for
  these param(s): threshold` in the codebase. The value is persisted in
  `settings.json` with `serde(default)` so existing files continue to
  load. Both the focus-view filter (`MediaFocusDialog`) and the
  similarity-pairs finder (`SimilarPairsDialog`) now read the stored
  setting instead of the compile-time constant.

- **Working ffmpeg re-download** (RFC 005). The "Get" button in
  Settings → AI now downloads and unpacks the ffmpeg binary using the
  same GitHub CDN source as the first-run setup. Status is shown
  inline ("Downloading ffmpeg…", "ffmpeg is ready.", or an error
  message) using the same component pattern as the clip "Load" button.
  `VideoEngine::download_and_install()` async helper added to the
  sidecar crate.

---

## [0.25.0]

### Added

- **Cache control page** (RFC 004; external design in
  `docs/src/dev/design/`). A third side-nav page (🗃) with:

  A **per-directory table** of cached entries — directory path, file
  count (images + videos merged), total size, and the newest cached-at
  timestamp in absolute local time — sorted newest-first, with a
  case-insensitive path filter and a refresh button. The summary line
  always shows unfiltered totals.

  **Per-row clearing** (🗑) — removes that directory's database entries
  in both namespaces and deletes the generated thumbnail files.
  Non-recursive: each row is exactly one directory.

  An **add-directory form** — runs the existing indexing pipeline
  (thumbnails → embeddings) for an arbitrary directory without changing
  the Explorer's selection. The run is reflected in the table at start
  (⏳ row indicator, placeholder row for never-cached directories) and
  at finish (reload with final values). Explorer-initiated runs mark
  the table identically. The single-task rule is preserved: a new run
  aborts an in-flight one.

  Facade additions in `arama-cache`: `DirCacheSummary`,
  `summarize_by_dir()` on both readers (payload-free enumeration via
  `localcache::EntryInfo`), and `delete_in_dir()` on both writers.
  Four new spec tests cover grouping/aggregation, the empty cache,
  thumbnail deletion, and non-recursiveness. New workspace dependency:
  `chrono` (clock feature only) for local-time formatting.

---

## [0.24.0]

### Changed

- **Side-nav shell** (RFC 003).
  The header-mounted settings button and the collapsible aside rail are
  replaced by a snora `side_bar` nav rail with two icon buttons:

  **Explorer** (`📁`) — the default page. Renders the directory-input
  header (full width), the always-visible directory tree as the left
  tile, and the gallery as the right tile. `AppLayout.header` is no
  longer used; the header widget lives inside the Explorer page body so
  that it is absent when another page is active.

  **Settings** (`⚙`) — renders the full settings content (General,
  AI, File system, About tabs) directly in the body without a modal.
  Settings state (active tab, AI loading message) is preserved across
  page switches because the widget is a persistent field on `App` rather
  than a dynamically created dialog.

  The `Aside` open/close toggle is removed; the directory tree is always
  visible. The `Dialog::SettingsDialog` variant is removed from the
  dialog enum. `NavTo(NavPage)` is the new message for page switching.
  No new dependencies are required — the nav rail is built from the
  existing Lucide icon set and iced button primitives.

---

## [0.23.0]

### Added

- **Directory switch cancels indexing** — switching the active directory
  while thumbnail or embedding generation is in progress now aborts the
  running task immediately (via `Task::abortable` + per-file
  `yield_now`), then starts a fresh run for the new directory. Previously
  the switch was silently ignored until indexing finished.

### Changed

- **Cache engine replaced with `localcache` v0.20** (RFC 002).
  The in-house `file-feature-cache` engine is retired. `arama-cache`
  now uses `localcache` for all storage: one SQLite file, two namespaces
  (`image` / `video`), `MetadataThenFullHash` change detection, parallel
  reads via `ReadPool`, and v1 → v2 one-time migration on first launch.
  Thumbnail files are renamed from row-id–based to path-hash–based
  (`blake3(canonical_path)[..16].jpg`). Public API of `arama-cache` is
  unchanged; all consumers compile without modification.

- **ffmpeg download source** — Linux and Windows now download ffmpeg
  from `yt-dlp/FFmpeg-Builds` on GitHub CDN instead of
  `johnvansickle.com` (Linux) and `gyan.dev` (Windows). Both were
  personal servers with low throughput; GitHub CDN matches the speed of
  the HuggingFace model downloads. `ffmpeg-sidecar` is removed from the
  dependency tree entirely; extraction is now handled directly using
  `tar`/`xz2` (Linux) and `zip` (macOS / Windows).

- **Setup download throughput** — progress updates in the download
  stream were sent with `.await` on every HTTP chunk, stalling the
  transfer whenever the iced event loop was busy (most visible with
  ffmpeg on slow-chunk hosts). Changed to `try_send` (non-blocking,
  drops update when channel is full) and increased `BufWriter` capacity
  from 8 KB to 256 KB.

- **Release archive structure** — archives are now named and structured
  as `arama-vX.Y.Z.tar.gz` with a matching `arama-vX.Y.Z/` inner
  directory. Previously the inner directory retained the source folder
  name (`arama-0.21.0/`) regardless of the release version.

### Fixed

- **`SQLITE_CANTOPEN (14)` on first run** — `localcache` / SQLite does
  not create intermediate directories. The `.arama-cache/` directory is
  now created with `create_dir_all` before any engine or pool is opened.

- **`all_in_dir` / `all_in_dir_and_sub_dirs` with a file path** — the
  "find similar" dialog passes the currently focused media file's path to
  these queries. `localcache`'s `path_in_dir` expects a directory;
  passing a file path returned zero entries and caused an index-out-of-
  bounds panic. Both readers now resolve a file path to its parent
  directory automatically (`dir_of` helper).

- **`similar_pairs_dialog` panic on missing features** — `.expect` on
  `features` panicked when a cache entry existed (thumbnail generated)
  but embeddings had not yet been computed. Changed to `Option` chaining;
  such entries are silently skipped.

- **Settings button on header had no effect** — `SettingsNav` fired
  `Message::SettingsOpen` but `header/update.rs` returned `Task::none()`
  instead of propagating it as `Header::Event::SettingsOpen`. The app's
  handler (which opens the settings dialog) was never reached.

---

## [0.22.0]

### Changed

- **UI framework migrated to snora v0.8** (RFC 001).
  The hand-rolled iced `stack!` + `overlay` layout is replaced by
  `snora::AppLayout`. Dialogs (`MediaFocusDialog`, `SimilarPairsDialog`,
  `SettingsDialog`) are now presented via `snora::Dialog`. Context-menu
  backdrop and click-outside dismissal are handled by snora. Error
  notifications use the snora toast system (`ToastIntent::Error`,
  `ToastPosition::BottomEnd`) — the previous `eprintln!` placeholders are
  replaced with visible toasts. Header and footer heights are owned by
  their respective components.
