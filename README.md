# arama

[![License](https://img.shields.io/github/license/nabbisen/arama)](LICENSE)
[![crates.io](https://img.shields.io/crates/v/arama?label=rust)](https://crates.io/crates/arama)
[![Rust Documentation](https://docs.rs/arama/badge.svg?version=latest)](https://docs.rs/arama)
[![Dependency Status](https://deps.rs/crate/arama/latest/status.svg)](https://deps.rs/crate/arama)

**Find similar images and videos — entirely on your machine.**

---

## Overview

arama is a desktop GUI application that uses offline AI to locate
visually or aurally similar media files inside a chosen directory tree.
There is no cloud service, no account, and no data leaves your device.

- **Images** are compared by CLIP visual embeddings (cosine similarity).
- **Videos** are compared by a weighted combination of CLIP frame
  embeddings and wav2vec2 audio embeddings.

Embeddings and thumbnails are cached in a local SQLite database so each
directory only needs to be indexed once.

---

## Why / When

| You want to… | arama can… |
|---|---|
| Deduplicate a photo library | Surface near-duplicate pairs across a folder |
| Find all shots of the same scene | Browse visually similar images from a gallery click |
| Locate a video by its audio content | Match audio via wav2vec2 embedding similarity |
| Keep AI processing private | Run everything locally — no API key, no upload |

arama works best with a reasonably modern desktop (an Apple Silicon Mac
or a multi-core Linux/Windows machine). CPU-only inference is supported;
a discrete GPU is not required.

---

## Quick Start

### Prerequisites

- [Rust toolchain](https://rustup.rs/) (stable, 2024 edition)
- An internet connection for the one-time AI model download
  (a few hundred MB)
- For **video** only: a matching `ffmpeg` and `ffprobe` pair that you install
  yourself — see [ffmpeg for video](#ffmpeg-for-video) below

### Build and run

```sh
# Extract the root-layout source archive into its own directory
mkdir arama-X.Y.Z
tar xzf arama-X.Y.Z.tar.gz -C arama-X.Y.Z
cd arama-X.Y.Z

# Build and launch (release mode recommended for AI inference speed)
cargo run -p arama --release
```

Platform executable assets and `cargo install arama` are also available. See
the [installation guide](./docs/src/users/installation.md) for the supported
asset matrix and the differences between distribution routes.

The first launch opens a setup wizard that downloads the AI models:
- `openai/clip-vit-base-patch32` — CLIP model for image similarity
- `facebook/wav2vec2-base-960h` — audio model for video similarity

They are stored alongside the executable under `.arama-local/`. CLIP alone
completes setup, so you can start using image similarity immediately. Once the
models are in place, no further network access is required.

### ffmpeg for video

Video analysis needs a matching `ffmpeg` and `ffprobe` pair. **arama does not
download, install, bundle, or invoke a package manager to obtain them** on any
platform — you install them yourself, so their license and provenance stay
yours. For example, on macOS:

```sh
brew install ffmpeg
```

arama then discovers the pair on your `PATH` (plus the native Homebrew prefix
on macOS), or you can point it at a specific folder from **Settings → AI**.
Until a valid pair is found, video features stay unavailable and everything
else keeps working. See the
[installation guide](./docs/src/users/installation.md#video-prerequisite) for
per-platform detail.

---

## Design Notes

- **Offline-first.** All AI inference runs locally with
  [candle](https://github.com/huggingface/candle). No telemetry.
- **iced GUI.** Built on [iced](https://github.com/iced-rs/iced) 0.14
  with the [snora](https://github.com/nabbisen/snora) shell framework.
  Side-nav pages: **Explorer** (directory tree + gallery tiling view),
  **Cache** (per-directory cache management), and **Settings**.
- **localcache persistence.** Embeddings and thumbnails are stored in
  a two-namespace SQLite database via
  [localcache](https://github.com/nabbisen/localcache), keyed by file
  path. Re-indexing is triggered automatically when a file changes.
- **Similarity threshold.** Both image and video similarity default to
  **0.86** cosine similarity (dot product of unit-norm CLIP vectors).
- **Supported formats.** Images: `png jpg jpeg webp gif bmp`.
  Videos: `mp4`.

---

## More Detail

Full documentation lives in [`docs/src/`](./docs/src/) and is
structured for [mdBook](https://rust-lang.github.io/mdBook/).

| Audience | Start here |
|---|---|
| New users | [Installation](./docs/src/users/installation.md) · [First Run](./docs/src/users/first-run.md) · [Using arama](./docs/src/users/using-arama.md) |
| Contributors | [Architecture](./docs/src/dev/architecture.md) · [Workspace](./docs/src/dev/workspace.md) · [Workflow](./docs/src/dev/workflow.md) |
