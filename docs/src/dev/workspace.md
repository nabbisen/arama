# Workspace Structure

The Cargo workspace root is `Cargo.toml`. Member crates:

```
arama-0.vX.Y.Z/
├── app/                      # Binary crate — main entry point
├── crates/
│   ├── ai/                   # AI inference pipeline
│   ├── cache/                # Embedding and thumbnail cache facade
│   ├── engine/
│   │   └── sidecar/          # ffmpeg binary management
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
- `migrate_v1_if_present` — one-time migration from the legacy cache

All consumers use the public API in `crates/cache/src/lib.rs`; the
localcache engine details are an implementation concern.

### `crates/engine/sidecar`

Manages the ffmpeg sidecar binary: download URL selection (GitHub CDN
via `yt-dlp/FFmpeg-Builds`), archive extraction, and spawning ffmpeg
and ffprobe commands.

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
 ├── crates/ui/layout
 │    └── crates/ui/widgets
 │         └── crates/ai  (for similarity computation in dialogs)
 ├── crates/ui/main
 │    └── crates/ui/widgets
 └── env  (all crates depend on env)
```
