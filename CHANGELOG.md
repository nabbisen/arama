# Changelog

All notable changes to arama are documented here.
Releases follow the archive naming `arama-vX.Y.Z.tar.gz`.

---

## [Unreleased]

### Planned

- Relative-time rendering ("2 days ago") for the Cache page table —
  absolute local time is the default; relative rendering is a future
  small follow-up.
- Stop button on the Cache page's in-progress ⏳ row.

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
