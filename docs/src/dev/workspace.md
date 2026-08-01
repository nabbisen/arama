# Workspace Structure

The Cargo workspace root is `Cargo.toml`. Member crates:

```
arama-X.Y.Z/
├── app/                      # Binary crate — main entry point
├── crates/
│   ├── ai/                   # AI inference pipeline
│   ├── cache/                # Embedding and thumbnail cache facade
│   ├── i18n/                 # Locale tables and t() translation function
│   ├── theme/                # Snora Design token-driven button styles
│   ├── engine/
│   │   └── sidecar/          # external ffmpeg discovery and probing
│   └── ui/
│       ├── layout/           # Shell layout (aside, header, footer)
│       ├── main/             # Gallery, setup wizard, core views
│       └── widgets/          # Reusable widgets (dir tree, dialogs)
├── env/                      # Environment constants and path helpers
├── docs/src/                 # mdBook documentation (this file)
├── rfcs/                     # Design documents (lifecycle per RFC 000)
├── CHANGELOG.md
├── NOTICE
├── README.md
└── version.sh                # Workspace-wide version bump helper
```

## Crate responsibilities

### `app`

The iced application binary. Owns `App` (the top-level state struct),
`Message`, `NavPage`, and the `update` / `view` / `subscription`
implementations. Depends on every other crate.

Key files:
- `app/src/core.rs` — `App` struct, `NavPage` enum, `Dialog` enum
- `app/src/core/update.rs` — all message handlers
- `app/src/core/view.rs` — snora `AppLayout` composition

### `crates/ai`

Offline AI inference. Contains:
- `pipeline/encode/image/` — CLIP image encoder and cosine similarity
- `pipeline/encode/audio/` — wav2vec2 audio encoder
- `pipeline_manager/` — `VideoSimilarityPipeline` (frame + audio
  sampling, parallel encoding, score weighting)
- `model/` — model container definitions and HuggingFace download
  metadata (`clip`, `wav2vec2`)
- `config/video_similarity_config.rs` — sampling timestamps and score
  weights

### `crates/cache`

The `arama-cache` facade over `localcache`. Exposes:
- `ImageCacheWriter` / `ImageCacheReader`
- `VideoCacheWriter` / `VideoCacheReader`
- `CacheMaintenance` — cache footprint measurement and explicit manual
  pruning

All consumers use the public API in `crates/cache/src/lib.rs`; the
localcache engine details are an implementation concern.

### `crates/engine/sidecar`

Owns the validated ffmpeg/ffprobe toolchain authority. Linux and Windows use
the same external-only policy as macOS: arama discovers a user-installed
same-directory pair from `PATH`, a selected directory, or the native Homebrew
prefix on macOS, and requires matching version tokens. It does not acquire or
publish executables.
See [Security Boundaries](./security.md).

### `crates/ui/layout`

The application shell: `Header`, `Aside`, `Footer`. These are layout
components without AI or cache dependencies. The `Header` holds the
directory input and similarity-pairs button. The `Aside` holds the
`DirTree` widget. The `Footer` holds file counts and the thumbnail-size
slider.

### `crates/ui/main`

The gallery and setup wizard. `Gallery` renders the thumbnail grid and
manages selection state. `Setup` drives the first-run downloader.

### `crates/ui/widgets`

Self-contained reusable widgets:
- `DirTree` — interactive directory tree with processing indicators
- `ContextMenu` — right-click menu
- `dialog/media_focus_dialog` — similar media focus view
- `dialog/similar_pairs_dialog` — near-duplicate pairs finder
- `dialog/settings_dialog` — tabbed settings panel (reused as both a
  page widget and the Settings page component)

### `crates/i18n`

`arama-i18n` provides the runtime translation function `t(key) -> String`,
`set_locale(Locale)`, `current_locale()`, and the `Locale` enum (`En`,
`Ja`). The active locale is stored in a global `AtomicU8`. English and
Japanese translation tables live in `en.rs` and `ja.rs` as static
`match` expressions. All UI crates depend on this crate.

### Recoverable error display policy

RFC 017 classifies recoverable user-visible failures into four tiers:
fatal startup errors, blocking view errors, recoverable action errors,
and developer diagnostics. Blocking view errors, such as Cache page
reload failures, should render inline near the stale or unavailable
data. Recoverable user actions, such as settings save failures, should
use app toasts. Static invariant failures may stay stderr-only when the
fallback remains truthful and safe.

RFC 018 applies that policy to AI/video indexing. Fatal setup remains an
error toast when no requested media work can proceed. Per-file video
decode/extraction failures and cache write failures are accumulated into
an indexing report so the app can warn once and continue unrelated
files. Video frame and audio modalities are independently valid; partial
entries compare only over shared valid modalities.

RFC 019 applies the same boundary discipline to startup. Failures that prevent
iced from opening a shell are returned as the top-level `iced::Result`.
Failures that still permit a truthful shell, such as local setup preflight
failure or an invalid saved root directory, recover with startup toasts and
fallback state. Developer invariants may keep `expect()` only when the fallback
would not be meaningful.

### `crates/theme`

`arama-theme` adopts the Snora Design system (RFC 010). It holds the
active design tokens globally (`Tokens::light()` by default) and exposes
drop-in iced button style functions (`primary`, `ghost`, `secondary`,
`danger`) with verified WCAG AA contrast. The app and UI crates depend on
it for button styling.

### `env`

Shared constants and path helpers used across all crates:
- Directory paths (`.arama-local/`, `.arama-cache/`)
- Media extension allowlists
- Similarity thresholds and UI size limits
- Settings model (`Settings` struct, `CacheLookupStrategy`)

## Dependency graph (simplified)

```
app
 ├── crates/ai
 │    └── crates/engine/sidecar
 ├── crates/cache
 ├── crates/i18n       (all UI crates depend on this)
 ├── crates/ui/layout
 │    └── crates/ui/widgets
 │         └── crates/ai  (for similarity computation in dialogs)
 ├── crates/ui/main
 │    └── crates/ui/widgets
 └── env  (all crates depend on env)
```
