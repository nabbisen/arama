# Settings

Click the **⚙** icon in the side nav to open the Settings page. It
has four tabs.

## General

| Setting | Description | Default |
|---|---|---|
| **Include image** | Index image files (png, jpg, jpeg, webp, gif, bmp) | On |
| **Include video** | Index video files (mp4) | On |
| **Sub-dir depth** | How many subdirectory levels to scan (0 = current directory only, 1 = one level deep, 2 = two levels) | 0 |
| **Similarity** | Cosine-similarity threshold used by the focus view and the similarity pairs finder. Range 0.50–1.00; higher = stricter (fewer, more exact matches). | 0.86 |
| **Language** | Display language. EN (English) or 日本語 (Japanese). Takes effect immediately with no restart. | EN |
| **Theme** | Application theme: Light, Dark, High contrast light, or High contrast dark. Takes effect immediately with no restart. High-contrast applies fully to arama's own controls, and the six core palette roles are mapped into standard iced widgets such as text inputs, sliders, scrollbars, and checkboxes. | Light |

Changes to media type or subdirectory depth take effect immediately:
the currently selected directory is re-indexed with the new parameters.

If arama cannot load saved settings at startup, it shows a warning and
uses default settings for that session. If a later settings save fails,
the current in-memory setting still takes effect immediately, but arama
shows an error because the change may not persist after restart.

## AI

Shows the status of the two AI models and the ffmpeg binary:

- If a model shows **"ready"**, it is loaded and available for inference.
- If a model is missing, a **Load** button appears. Click it to
  download the model from HuggingFace. This is the same download that
  runs automatically during first launch.
- On every platform, a missing ffmpeg pair is an external prerequisite. Install
  a pair through a source you trust, then use **Re-check** or select its
  directory. Re-check only discovers locally installed tools; arama does not
  run a package manager or download an executable.

Wav2vec2 and ffmpeg are optional for image-only use. A compatible pair must
contain both `ffmpeg` and `ffprobe` in one candidate directory and both must
report the same release/build token.

This tab is useful after a clean install or if arama's models directory (see
[Data locations](installation.md#data-locations)) was moved or deleted.

## File system

| Item | Description |
|---|---|
| **Disk usage** | Available / total disk space on the volume containing arama's models directory |
| **Cache delete** | Remove arama's entire cache directory (thumbnails + embeddings). The next directory selection will re-index from scratch. |

The Cache delete button is disabled when the cache directory does not
exist.

## About

Shows a link to the project repository on GitHub.
