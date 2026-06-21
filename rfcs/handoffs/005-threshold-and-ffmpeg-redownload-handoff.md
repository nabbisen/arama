# Handoff — RFC 005: Configurable similarity threshold + ffmpeg re-download

**RFC.** [`rfcs/done/005-threshold-and-ffmpeg-redownload.md`](../done/005-threshold-and-ffmpeg-redownload.md)
**Shipped in.** v0.26.0

Two companion `Settings` improvements, each resolving a `// todo` in the
codebase.

---

## 1. Implementation Handoff

### (a) Configurable similarity threshold
- Add `similarity_threshold: f32` to `Settings` (default
  `MIN_IMAGE_SIMILARITY` = 0.86, via `#[serde(default = "…")]`). The
  `MIN_IMAGE_SIMILARITY` / `MIN_VIDEO_SIMILARITY` constants stay in `env` as
  the defaults — not removed.
- Labeled slider (0.50–1.00, step 0.01) in Settings → General, below the
  sub-dir depth control.
- Thread the value through `MediaFocusDialog::new` and
  `SimilarPairsDialog::new`; use it in place of the constants in both the
  focus-view filter (`similar_media.rs`) and the pairs finder.
- Persist in `settings.json`; `SimilarityThresholdChanged` bubbles
  GeneralSettings → SettingsDialog → App.

### (b) ffmpeg re-download
- Wire the previously dead `Settings → AI → Get` button to
  `Message::GetFfmpegStart`, a background `Task::perform` reusing
  `VideoEngine::download_url()` + `unpack_archive()`.
- Status string on `AiSettings` ("Downloading ffmpeg…" / "ffmpeg is ready." /
  error). No streaming progress bar — the single-task model the clip "Load"
  button uses is sufficient.

---

## 2. Task Breakdown / PR Plan

Two independent PRs:

### PR 1 — Threshold
1. `Settings.similarity_threshold` + serde default.
2. Slider in GeneralSettings; `SimilarityThresholdChanged` through the dialog
   chain to the app; save.
3. Thread through both dialog constructors; replace the constants at the two
   computation sites.

### PR 2 — ffmpeg Get button
4. `GetFfmpegStart` message + `Task::perform` download; status string on
   `AiSettings`.

---

## 3. Acceptance / QA Checklist

### Automated
- [ ] `cargo check --workspace` — zero errors, zero warnings.
- [ ] Existing `settings.json` without `similarity_threshold` loads as 0.86.

### Manual — threshold
- [ ] The slider shows the current value (default 0.86) and adjusts in 0.01
      steps within 0.50–1.00.
- [ ] Lowering the threshold yields more focus-view / pairs matches; raising
      it yields fewer.
- [ ] The value persists across restart and is reflected in both the focus
      view and the pairs finder.

### Manual — ffmpeg
- [ ] With ffmpeg absent, Settings → AI shows the "Get" button; pressing it
      downloads and unpacks, and the status transitions to "ffmpeg is ready."
- [ ] A download failure surfaces as an error status string, not a crash.
- [ ] After a successful Get, video analysis works without restart.
