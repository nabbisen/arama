# RFC 005 — Configurable similarity threshold + ffmpeg re-download

**Status.** Implemented (v0.26.0)
**Tracks.** Two companion improvements to `Settings`:
(a) the similarity threshold used by the focus view and similarity
pairs finder is currently hardcoded at 0.86 and is explicitly marked
`// todo ui sliders for these param(s): threshold` in the codebase;
(b) the "Get" button in Settings → AI has `// todo: on_press()` and
has never been wired.
**Touches.** `env/src/config/settings.rs`, `crates/ui/widgets/`
(GeneralSettings, AiSettings, MediaFocusDialog, SimilarPairsDialog),
`crates/engine/sidecar/` (new async download helper), `app/src/core/`
(update.rs — dialog construction, settings bubble-up).

## Summary

### (a) Similarity threshold

The constants `MIN_IMAGE_SIMILARITY = 0.86` and
`MIN_VIDEO_SIMILARITY = 0.86` are hardcoded in `arama_env`. Both the
focus-view filter (`similar_media.rs`) and the pairs finder
(`similar_pairs_dialog.rs`) read them at compile time. There is no way
to tune the search without recompiling.

A single `similarity_threshold: f32` field is added to `Settings`
(defaulting to 0.86). A labeled slider (range 0.50–1.00, step 0.01)
is added to Settings → General, below the existing subdirectory-depth
control. The threshold is persisted in `settings.json`, threaded
through `MediaFocusDialog::new` and `SimilarPairsDialog::new`, and
used in place of the constants in both search computations.

The constants `MIN_IMAGE_SIMILARITY` / `MIN_VIDEO_SIMILARITY` remain
in `env` as the default values; they are not removed.

### (b) ffmpeg re-download

`Settings → AI → Get` fires `Message::GetFfmpegStart`, which runs a
background `Task::perform` that downloads and unpacks the ffmpeg binary
using the existing `VideoEngine::download_url()` and
`VideoEngine::unpack_archive()` infrastructure. A status string on the
`AiSettings` component shows progress ("Downloading ffmpeg…", "ffmpeg
is ready.", or the error message). No streaming progress bar — the
simple single-task model used by the clip "Load" button is sufficient.

## Design

### Settings struct (env)

```rust
pub struct Settings {
    // ...existing fields...
    pub similarity_threshold: f32,   // default: MIN_IMAGE_SIMILARITY (0.86)
}
```

Backward-compatible: `serde(default)` is applied so that existing
`settings.json` files without the field continue to load cleanly.

### Slider (GeneralSettings)

```
Sub-dir depth    ▼ 0 ▲

Similarity  0.50 ─────●─── 1.00   0.86
```

The slider emits `Message::SimilarityThresholdChanged(f32)` (truncated
to two decimal places on display, stored at full f32 precision).
The message path to the app:
`GeneralSettings::Message` → `SettingsDialog::Message` → `App`.

### MediaFocusDialog

```rust
pub fn new<T: Into<PathBuf>>(
    path: T,
    cache_lookup_strategy: CacheLookupStrategy,
    similarity_threshold: f32,
) -> Self
```

`similar_media()` filters by `self.similarity_threshold` instead of
`MIN_IMAGE_SIMILARITY`.

### SimilarPairsDialog

```rust
pub fn new(
    dir_node: DirNode,
    cache_lookup_strategy: Option<CacheLookupStrategy>,
    similarity_threshold: f32,
) -> Self
```

The async pairs computation passes `self.similarity_threshold` to
`find_similar_pairs`.

### AiSettings — ffmpeg download

New messages:

```rust
pub enum Message {
    LoadStart,
    Loaded(Option<String>),
    GetFfmpegStart,         // new
    FfmpegGot(Option<String>), // new
}
```

`GetFfmpegStart` handler spawns:

```rust
Task::perform(
    async {
        VideoEngine::download_and_install().await
            .err().map(|e| e.to_string())
    },
    Message::FfmpegGot,
)
```

`VideoEngine::download_and_install()` (new async function in sidecar):
1. `download_url()` → url string
2. `reqwest::get(&url).await?.bytes().await?` → bytes
3. `tokio::fs::write(download_dest_path(), bytes).await?`
4. `unpack_archive()?`

Buffers the binary (~80 MB) in memory — acceptable for a one-time
re-installation. Uses `reqwest` (already a workspace dep with the
`stream` feature) and `tokio::fs` (already a workspace dep).

## Touches in detail

| File | Change |
|---|---|
| `env/src/config/settings.rs` | Add `similarity_threshold: f32`, `#[serde(default = "default_threshold")]` |
| `crates/ui/widgets/src/dialog/settings_dialog/tab/general_settings/` | Add slider + `SimilarityThresholdChanged(f32)` message |
| `crates/ui/widgets/src/dialog/settings_dialog/` | Bubble `SimilarityThresholdChanged` up |
| `crates/ui/widgets/src/dialog/media_focus_dialog/` | Add `similarity_threshold` field; update `new()` and `similar_media()` |
| `crates/ui/widgets/src/dialog/similar_pairs_dialog.rs` | Add `similarity_threshold` field; update `new()` and async computation |
| `crates/engine/sidecar/src/media/video/video_engine.rs` | Add `download_and_install()` async fn |
| `crates/engine/sidecar/Cargo.toml` | Add `reqwest` + `tokio` direct deps |
| `crates/ui/widgets/src/dialog/settings_dialog/tab/ai_settings/` | Add `GetFfmpegStart` / `FfmpegGot`; wire "Get" button |
| `app/src/core/update.rs` | Pass threshold to both dialog constructors; handle `SimilarityThresholdChanged` |
| `docs/src/users/settings.md` | Document threshold slider |

## Open questions

None.
