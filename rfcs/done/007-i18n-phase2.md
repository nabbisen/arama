# RFC 007 — i18n Phase 2 sweep

**Status.** Implemented (v0.28.0)
**Tracks.** The Phase 2 string sweep deferred in RFC 006: gallery,
focus dialog, similar-pairs dialog, setup wizard, and the header
folder-picker button.
**Touches.** `crates/i18n/src/{en,ja}.rs` (new keys), six view files,
one `state_name()` function, Japanese comments in gallery.

## Summary

RFC 006 shipped the i18n infrastructure and translated the Settings and
Cache pages. This RFC completes the sweep across the remaining surfaces:

| View file | Strings |
|---|---|
| `setup/view.rs` | "Download", "Skip", "Not enough space…" |
| `downloader/view.rs` | Status labels, component names, disk-space text |
| `media_focus_dialog/view.rs` | "Cache lookup strategy", "Close" |
| `similar_pairs_dialog/view.rs` | "No valid pairs." |
| `header/dir_nav/view.rs` | "Folder" (file-picker button) |
| `gallery/view.rs` | "No file to render." (empty state) |

## Additional fixes shipped with this RFC

- **Typo**: "No enough space on device for downloader." →
  corrected English and then replaced with `t("setup.no_space")`.
- **Typo**: `panic!("Unknown donwload config")` →
  `eprintln!` + graceful fallback name (removes the panic from a
  non-fatal path in the UI).
- **Japanese comments** in `gallery/view.rs` translated to English
  (project convention: all comments in English).

## New translation keys

```
setup.download           setup.skip
setup.no_space
setup.item.clip          setup.item.wav2vec2       setup.item.ffmpeg
setup.item.size_unknown
setup.status.missing     setup.status.downloading  setup.status.ready
setup.status.error
setup.not_ready          setup.ready
setup.download_into
setup.disk_space         setup.disk_gb_avail       setup.disk_gb_total
focus.strategy           focus.close
pairs.no_valid
header.folder
gallery.empty
```

## Open questions

None.
