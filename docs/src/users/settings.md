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
| **Theme** | Application theme: Light, Dark, High contrast light, or High contrast dark. Takes effect immediately with no restart. High-contrast applies fully to arama's own controls (buttons and future design-system widgets). Standard iced widgets (text inputs, sliders, scrollbars, checkboxes) follow the matching base Light or Dark theme — a known limitation of how the 18 design tokens map onto iced's 6-field palette. | Light |

Changes to media type or subdirectory depth take effect immediately:
the currently selected directory is re-indexed with the new parameters.

## AI

Shows the status of the two AI models and the ffmpeg binary:

- If a model shows **"ready"**, it is loaded and available for inference.
- If a model is missing, a **Load** button appears. Click it to
  download the model from HuggingFace. This is the same download that
  runs automatically during first launch.
- If ffmpeg is missing, a **Get** button appears. Click it to download
  the ffmpeg binary.

This tab is useful after a clean install or if the `.arama-local/`
directory was moved or deleted.

## File system

| Item | Description |
|---|---|
| **Disk usage** | Available / total disk space on the volume containing the executable |
| **Cache delete** | Remove the entire `.arama-cache/` directory (thumbnails + embeddings). The next directory selection will re-index from scratch. |

The Cache delete button is disabled when the cache directory does not
exist.

## About

Shows a link to the project repository on GitHub.
