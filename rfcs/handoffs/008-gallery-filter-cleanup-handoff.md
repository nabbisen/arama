# Handoff — RFC 008: Gallery filename filter, AI debug cleanup, error handling

**RFC.** [`rfcs/done/008-gallery-filter-cleanup.md`](../done/008-gallery-filter-cleanup.md)
**Shipped in.** v0.29.0

Three independent improvements bundled into one release.

---

## 1. Implementation Handoff

### (a) Gallery filename filter
A `filter: String` field on `Gallery`, a filter row above the thumbnail grid
(text input + clear ✕ + "N of M" count), case-insensitive substring match on
the **filename component only** (`filename_matches` helper). Directory group
labels are preserved only when a group has ≥1 match. The filter resets on
directory change via `Gallery::clear_filter()` called from
`App::on_dir_changed`.

### (b) AI debug cleanup
Remove the `// todo: delete debugger` `println!` blocks from
`video_extractor.rs`, `clip_encoder.rs`, `video_similarity_pipeline.rs`.
Legitimate frame-extraction diagnostics are kept but moved to `eprintln!`
(prefixed `arama:`).

### (c) Error-handling sweep
Replace `// todo: error handling` panic/silent sites:
- `Setup::default()` failure → startup error toast + `Setup::fallback()`
  (skips the wizard) instead of panic.
- `dir_node()` allowlist failure → unfiltered walk fallback, not panic.
- Thumbnail cache writer → `let Ok(writer) = … else { return vec![]; }`.
- `SimilarPairsDialog` open with no directory → error toast guard, not
  `unwrap()`.

### Watch out for
- `iced` `button(...)` does not accept `String`; wrap label strings in
  `text(...)`.
- The regex-removal of `println!` blocks is fragile across multi-line
  macros — remove by matching the `// todo` comment + paren-depth, not a
  blanket regex (a naive regex truncated a multi-line `println!` during
  development).

---

## 2. Task Breakdown / PR Plan

Three independent PRs (the three parts do not depend on each other):

### PR 1 — Gallery filter
1. `gallery/message.rs`: `FilterChanged(String)`, `FilterClear`.
2. `gallery.rs`: `filter` field + `clear_filter()`.
3. `gallery/update.rs`: handle the two messages.
4. `gallery/view.rs`: filter row + filtered grid + `filename_matches`.
5. i18n: `gallery.filter.{placeholder,clear,count_of}` in en/ja.
6. `app/src/core/update.rs`: route `FilterChanged`/`FilterClear`; call
   `clear_filter()` in `on_dir_changed`.

### PR 2 — AI debug cleanup
7. Remove the three sets of debug `println!`; convert frame diagnostics to
   `eprintln!`.

### PR 3 — Error handling
8. `Setup::fallback()` + `App::new` toast path; `dir_node` fallback; cache
   writer early-return; `SimilarPairsDialog` None-guard; remove stale
   `// todo` comments in `media_focus_dialog`.

---

## 3. Acceptance / QA Checklist

### Automated
- [ ] `cargo check --workspace` — zero errors, zero warnings (the debug-print
      removal should also clear any unused-import warnings).
- [ ] No `println!` remains in the three AI files (`grep`).
- [ ] No `// todo: error handling` or `// todo: delete debugger` remains.

### Manual — filter
- [ ] Typing in the filter narrows the gallery; "N of M" reflects matches.
- [ ] Match is on filename only (`sunset` matches `IMG_sunset.jpg`, not the
      `/Photos/` directory segment).
- [ ] Clearing (✕) or emptying the box restores all files.
- [ ] Switching directory resets the filter.

### Manual — error handling
- [ ] With a deliberately broken setup precondition, the app starts and shows
      an error toast rather than panicking.
- [ ] Opening "similarity pairs" with no directory selected shows
      "Select a directory first." rather than crashing.
- [ ] Caching a directory whose thumbnail files were externally deleted does
      not panic (entries are skipped).

### Regression
- [ ] Terminal output during inference is quiet (no debug spam).
