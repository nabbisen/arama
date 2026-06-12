# RFC 008 — Gallery filename filter, AI debug cleanup, error handling

**Status.** Implemented (v0.29.0)
**Tracks.** Three independent improvements shipped as one release:
(a) a filename search filter for the gallery;
(b) removal of debug `println!` calls from the AI pipeline crates;
(c) replacement of `// todo: error handling` sites in the app with
proper toast notifications.
**Touches.** `crates/ui/main/src/core/views/gallery/`,
`crates/i18n/`, `app/src/core{,.rs/update.rs}`,
`crates/ai/src/pipeline/extract/video_extractor.rs`,
`crates/ai/src/pipeline/encode/image/clip_encoder.rs`,
`crates/ai/src/pipeline_manager/video_similarity_pipeline.rs`.

---

## (a) Gallery filename filter

### Motivation

With hundreds of thumbnails in the gallery, locating a specific file
requires scrolling. A lightweight in-gallery filter solves this without
a separate search page.

### Design

A `filter: String` field is added to `Gallery`. The gallery `view()`
renders a filter row above the thumbnail grid:

```
[ Filter by filename… ]  ✕   42 of 312
```

- The text input emits `Gallery::Message::FilterChanged(String)`.
- The `✕` button clears the filter.
- The count `N of M` is shown when the filter is non-empty; hidden
  otherwise (avoids UI clutter when browsing normally).
- Filtering is case-insensitive substring match on the **filename**
  component only (not the full path), so "sunset" matches
  `/Photos/2024/IMG_sunset_042.jpg`.
- The directory label row is preserved: groups remain visible as long
  as they have matching files. An empty group is omitted.
- The empty state remains `t("gallery.empty")` regardless of whether
  the filter is active or the directory has no files.
- The filter resets automatically when the selected directory changes:
  `App::on_dir_changed` calls `self.gallery.clear_filter()`.

### New translation keys

```
gallery.filter.placeholder  →  "Filter by filename…"  /  "ファイル名でフィルター…"
gallery.filter.clear        →  "✕"  (same in both locales)
gallery.filter.count        →  "of"  /  "件中"
```

### Message changes

```rust
// gallery/message.rs
pub enum Message {
    ImageCellMessage(image_cell::message::Message),
    CursorExit,
    FilterChanged(String),   // new
    FilterClear,             // new
}
```

---

## (b) AI pipeline debug cleanup

The following `println!()` calls were added during development and are
now marked `// todo: delete debugger`. They output to stdout in
production builds, polluting terminal output and slightly slowing
inference on large directories:

| File | What it prints |
|---|---|
| `video_extractor.rs` | Video duration, per-frame extraction results, summary counts |
| `clip_encoder.rs` | Progress counter during batch encoding |
| `video_similarity_pipeline.rs` | Intermediate scoring values, pipeline progress |

All are removed unconditionally. The surrounding logic is untouched.

---

## (c) Error handling sweep

Four sites annotated `// todo: error handling` in `app/src/core.rs`
and `app/src/core/update.rs`. Each becomes a startup toast or an early
return with a toast rather than panicking or silently dropping the error.

| Site | Current | After |
|---|---|---|
| `App::new` — `Setup::default().expect("Failed to setup preparation")` | panic | startup error toast; app continues in "setup unavailable" degraded state |
| `App::new` — inline `// todo: error handling` on model-check | silently ignored | startup warning toast if models cannot be queried |
| `update.rs:48` — within `ThumbnailCacheFinished` handler | silently ignored | toast per failed file |
| `update.rs:243` — within `EmbeddingCacheFinished` handler | silently ignored | toast per failed file |

The two `// todo` comments on working code in `media_focus_dialog`
(which are placeholder notes, not actionable errors) are removed.

---

## Open questions

None.
