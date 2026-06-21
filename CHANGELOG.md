# Changelog

All notable changes to arama are documented here.
Releases follow the archive naming `arama-vX.Y.Z.tar.gz`.

---

## [Unreleased]

### Planned

- Relative-time rendering ("2 days ago") for the Cache page table.

---

## [0.36.1]

### Fixed

- **Aside tree panel width collapses on deep directories; header path not
  synced after aside click.**

  *Width / scroll (Issue 1):* The previous fix set a hard 200 px width on
  the aside column, which clips long paths in deeply-nested directories with
  no way to scroll horizontally. The outer column now uses
  `Length::FillPortion(1)` so the panel scales responsively with the window
  (gallery implicitly takes the remaining space). The tree widget is wrapped
  in a second `scrollable` with `Direction::Both` to add a horizontal
  scrollbar when paths overflow the panel width.

  *Header sync / gallery update (Issue 2):* After clicking a directory in
  the aside tree, `on_dir_changed` correctly started the cache pipeline and
  updated the gallery, but the header path input remained frozen at the
  previous directory because `DirNav` had no external setter.
  `DirNav::set_path` and `Header::set_path` were added; `on_dir_changed`
  now calls `self.header.set_path(…)` so the input stays in sync regardless
  of how navigation was triggered (header submit, file-picker, or aside click).

- **Explorer aside tree: focus current directory; no auto-close** (RFC 014 follow-up).
  Three UX corrections:

  *No auto-close on selection:* The pane stays open after the user picks a
  directory so they can navigate multiple subdirectories before closing manually.

  *Parent directories visible:* `DirectoryTree` is rooted at the filesystem
  root (`/` on Unix, `C:\\` on Windows). When the pane opens or navigation
  changes, the tree cascades `Toggled` events from root down to the current
  directory, revealing the full ancestor chain. `Aside` holds an `expand_queue`
  (outermost-first) drained one level per `Loaded` event.

  *Current directory selected and scrolled into view:* When the cascade
  completes (`ExpandDone`), `finish_expand` issues `Selected(target, Replace)`
  to highlight the current directory, then calls `widget::operation::snap_to`
  on a named outer `scrollable` (`aside-tree-scroll`) with `RelativeOffset::END`
  to scroll the viewport down to the selected row.

- **Explorer aside tree: always-visible panel replaced with toggle** (RFC 014).
  The previous always-on panel caused scroll/width problems (fixed width clipped
  deep paths; `Direction::Both` scrollbars were visually confusing). The panel
  is now toggled open/closed via a button left of the header path input
  (`icon_panel_left_open` / `icon_panel_left_close`). Selecting a directory in
  the tree closes the pane automatically. Gallery has full width by default.
  `Header::set_path` / `DirNav::set_path` keep the header input in sync after
  an aside-driven navigation.

- **Aside (directory tree) invisible on Explorer view** (v0.36.0 initial fix).
  `Aside::view()` used `column![tree].width(Length::Shrink)`, which collapses
  to zero width before the first async directory scan completes.
  Fixed by giving the column an explicit width; superseded above by the
  toggle approach.

---

## [0.36.0]

### Changed

- **ELOC splits** (RFC 013). Two files that exceeded the 500 ELOC threshold
  are split along natural logical seams; every `.rs` file is now under 300 ELOC.

  `app/src/core/update.rs` (543 ELOC) becomes a 35-ELOC router that delegates
  to three sub-files under `update/`: `cache.rs` (pipeline handlers and dir
  helpers), `component.rs` (Setup, Gallery, Header, Aside, Footer, and dialog
  delegation), and `ui.rs` (nav, toast, cursor, and dialog-close housekeeping).

  `crates/cache/tests/integration_tests.rs` (615 ELOC) becomes a 0-ELOC
  module doc, with tests split into four sibling files: `helpers.rs` (shared
  fixtures), `image.rs` (9 image-namespace tests), `video.rs` (7
  video-namespace tests), `cross.rs` (11 cross-namespace / session / parallel
  / directory tests). `crates/cache/Cargo.toml` gains `autotests = false`
  and four explicit `[[test]]` entries so `helpers.rs` is not compiled as a
  standalone binary.

### Fixed

- **Stale test assertions in `arama-ai`** (`video_similarity_config` tests).
  `test_1hour` asserted `len == 12` and `test_90s` checked all consecutive
  gaps ≥ 20 s, both written against an earlier algorithm that lacked
  `head_fixed_anchors_secs`. The current design intentionally keeps fixed
  head anchors (3 s, 9 s, 15 s) regardless of gap; the correct count for a
  1-hour video is 13. Tests rewritten to validate the design spec: fixed
  anchors are always present, and only non-fixed consecutive pairs must
  respect `min_sample_gap_secs`.
- **Dead-code and unused-import warnings** in `arama-cache` integration
  tests. Each sibling test binary (`image.rs`, `video.rs`, `cross.rs`)
  includes `helpers.rs` via `#[path]` but uses only a subset of its items.
  Added `#[allow(dead_code)]` on the `mod helpers` declaration in each
  binary; removed the unused `use std::path::Path` import from `cross.rs`.

---

## [0.35.0]

### Changed

- **Single-source workspace versioning + metadata inheritance** (RFC 012).
  The version now lives only in `[workspace.package].version`; every
  member inherits `version`, `authors`, `repository`, `license`,
  `edition`, `rust-version`, `categories`, and `keywords` via
  `{ workspace = true }`, keeping only its own `description` and `readme`.
  Internal crates in `workspace.dependencies` carry an explicit `version`
  alongside `path` so the full crate graph is publishable (required for
  deps.rs and docs.rs). `version.sh` updates both locations atomically.
- **Workspace `repository` corrected** from `.../orbok` to `.../arama`.
- **`version.sh` simplified** to a jq-free script that updates both
  `[workspace.package].version` and the internal `workspace.dependencies`
  version fields in a single command.
- **Release docs corrected** (`docs/src/dev/release.md`): release archives
  now use a no-parent-directory layout (project files at the archive
  root), and the version-bump step is the single `version.sh --update`
  command.

### Removed

- **Orphan crate `arama-storage`** (`crates/engine/storage`). The
  pre-`localcache` storage engine superseded by RFC 002 (v0.23.0). It had
  been outside the build graph — absent from `members`,
  `workspace.dependencies`, every dependency list, and `Cargo.lock`, with
  no source references — and is now deleted.

### Fixed

- **`pt2safetensors` 0.1.2 build break resolved** by upgrading to 0.1.3.
  0.1.2 declared `safetensors` with `default-features = false` but called
  `serialize_to_file` (gated behind `std` since safetensors 0.5.0), and
  resolved a different safetensors minor version (0.8) than `candle-core`
  0.10 (0.7), making the `View` trait incompatible across crate instances.
  0.1.3 pins `candle-core = "0.10"` and `safetensors = { version = "0.7",
  features = ["std"] }`; the workspace constraint is updated to `"0.1.3"`.
  See `rfcs/notes/dep-fix-pt2safetensors.md` for the full analysis.

- **Incorrect `readme` paths** in the `arama`, `arama-ai`, and
  `arama-ui-layout` manifests previously pointed at non-existent files;
  every member now resolves `readme` to the root `README.md`.

---

## [0.34.0]

### Added

- **Snora recipe: `Theme::custom` from design tokens** (`rfcs/notes/snora-recipe-theme-custom.md`). RFC-033 nine-section recipe documenting how to map a `Tokens` preset onto an iced `Theme::custom` so stock iced widgets track the active design preset. Covers the 6-role mapping, the expansion caveat, call-site patterns, and customization points. Intended as a contribution to the snora recipe collection; seeded from arama's implementation.
- **Smoke tests for `arama-i18n`** (`locale_round_trip`, `translation_and_fallback`).
  Cover locale switching, the current→English→raw-key fallback chain, and the
  `Locale` code/display-name accessors. The crate has zero heavyweight dependencies
  so the test binary is fast to build.
  `iced_test` was evaluated as a candidate for view-layer smoke tests but not
  adopted: its `Simulator` links the full iced rendering stack (wgpu, winit,
  wayland, tiny-skia) even for headless tests, making test builds prohibitively
  heavy with no proportionate benefit for a project whose testable logic lives
  outside the view layer.

### Changed

- **RFC 011 high-contrast caveat sharpened.** The explanation "iced 0.14 has no built-in high-contrast theme" replaced with the precise mechanism confirmed by the snora team: snora's 18-role `Palette` collapses to iced's 6-field `theme::Palette`, and iced's own palette-expansion algorithm cannot reproduce the hand-tuned HC values for the 12 roles that don't survive (`surface` variants, `*_text` on-colors, `border`, `focus`, `text_secondary/muted`). The "future RFC" framing corrected: a full-palette bridge is out of scope for snora by design; the future work is an arama-side `Theme::custom` task. Updated in RFC 011, the theme-setting handoff, and `docs/src/users/settings.md`.
- **`lucide-icons` 0.576.0 → 1.17.0.** The 20 removed icons are all
  brand/social-media icons (Twitter, GitHub, Figma, etc.); none are used
  in arama. The `iced` feature and all function signatures are unchanged.
  Workspace constraint updated to `"1"`. (Migration report:
  `rfcs/notes/dep-migration-lucide-icons.md`.)
- **`candle-core` / `candle-nn` / `candle-transformers` 0.9.2 → 0.10.2.**
  Zero items removed from any of the three crates. Every struct, trait,
  and function that arama-ai imports exists unchanged in 0.10.2. The two
  additions (`TokenizerFromGguf` in core, `remove_mean` in nn) are
  unrelated to arama's CLIP/wav2vec2 pipeline. Constraints in
  `arama-ai/Cargo.toml` updated to `"0.10"`. (Migration report:
  `rfcs/notes/dep-migration-candle.md`.)

---

## [0.33.0]

### Added

- **Application theme setting — light / dark / high-contrast** (RFC 011).
  A new Theme selector in Settings → General lets users choose among the
  four Snora Design presets. The choice is persisted in `settings.json`
  (`theme` field, `serde(default)` = light) and applied immediately with
  no restart.

  The switch moves three styling layers together: snora button tokens
  (and reserved container tokens) resolve from the active preset via
  `arama-theme`, and a new iced `.theme()` callback returns the matching
  base `Theme::Light` / `Theme::Dark` so the window background and all
  stock iced widgets track the preset. `arama-theme`'s global moved from
  a write-once `OnceLock` to a mutable `AtomicU8` (the same pattern as the
  i18n locale).

  High-contrast presets apply their full token set to arama's own
  controls; iced 0.14 has no built-in high-contrast base theme, so stock
  iced widgets fall back to the matching light/dark base — documented in
  the settings UI and as a named future RFC (a full `Tokens` →
  `Theme::custom` bridge).

  New `ThemePreset` enum lives in `arama-env` (pure data, GUI-free,
  alongside the other persisted setting enums); `arama-theme` maps it to
  tokens and the iced theme. Round-trip tests for the enum's discriminants
  and serde mapping added to `arama-env`.

### Changed

- **snora 0.25.0 → 0.25.1.** Additive re-export fix:
  `snora::design::contrast` now resolves through the facade. Drop-in; no
  source change required.

---

## [0.32.0]

### Changed

- **snora 0.18.1 → 0.25.0** (RFC 010). Drop-in for arama's existing usage —
  all `AppLayout` builder methods, `Toast`, `ToastIntent`, `ToastPosition`,
  `render`, `toast::subscription`, and `toast::sweep_expired` are unchanged.
  The one breaking change in the range (`Palette::roles()` made test-only in
  0.24.0) is not used by arama.

### Added

- **Adopt the Snora Design system for button styling** (RFC 010). snora
  0.25.0 ships an opt-in `design` feature: an iced-free, zero-dependency
  token crate (`snora-design`) plus an iced style bridge whose button
  colors are verified to meet WCAG AA contrast (≥4.5:1) across four
  built-in presets.

  - New `arama-theme` crate holds the active design tokens globally
    (the same pattern arama uses for i18n) and exposes drop-in button
    style functions (`primary`, `ghost`, `secondary`, `danger`) with
    iced's exact `fn(&Theme, button::Status) -> button::Style` shape.
  - arama's buttons migrate from iced's built-in styles to the
    token-driven equivalents: nav rail (active = primary, inactive =
    ghost), locale selector, cache-page stop button (danger), and
    setup skip button (secondary).
  - Initialised with `Tokens::light()` to match arama's default
    `Theme::Light`. A future light / dark / high-contrast setting can
    change only the initialisation, with no call-site churn.

  Migration analysis at `rfcs/notes/dep-migration-snora.md` (updated for
  the 0.18 → 0.25 range).

---

## [0.31.0]

### Changed

- **snora 0.8.0 → 0.18.0** (ten minor versions). Drop-in update —
  no source changes required. All `AppLayout` builder methods, `Toast`,
  `ToastIntent`, `ToastPosition`, `render`, `toast::subscription`, and
  `toast::sweep_expired` are present and signature-identical in 0.18.0.

  Notable changes across the skipped versions: `AppLayout` is marked
  `#[non_exhaustive]` (arama already used the builder, not struct
  literal); toast ordering fixed (newest toast now correctly appears
  closest to the anchor edge); `snora::keyboard::dismiss_on_escape`
  helper added (not yet used by arama); `Icon: PartialEq` added.

  Migration report at `rfcs/notes/dep-migration-snora.md`.

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
