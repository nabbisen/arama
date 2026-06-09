# Introduction

arama is a desktop application for finding similar images and videos
using offline AI. Select a directory, wait for the one-time indexing
pass, then click any file to see what is nearby in embedding space.
Everything runs locally — no cloud service, no account, no upload.

## How it works

When you select a directory, arama:

1. Generates a 224×224 JPEG thumbnail for every image and video file.
2. Encodes each image through **CLIP** (`clip-vit-base-patch32`) to
   produce a 512-dimensional feature vector.
3. For video files, also samples frames and audio segments, encoding
   each through CLIP and **wav2vec2** (`wav2vec2-base-960h`).
4. Stores everything in a local SQLite cache alongside the executable.

Similarity is the **cosine similarity** (dot product of unit-norm
vectors) between two feature vectors. The default threshold is **0.86**:
files scoring at or above that value are considered similar.

## Who this documentation is for

**New users** — start with [Installation](./users/installation.md) and
[First Run](./users/first-run.md), then [Using arama](./users/using-arama.md).

**Intermediate users** — the [Settings](./users/settings.md) reference
covers every tunable parameter; the [FAQ](./users/faq.md) addresses
common questions about performance and file support.

**Contributors and maintainers** — the developer section covers the
codebase [architecture](./dev/architecture.md), workspace layout,
development conventions, the RFC design process, testing, and how
releases are packaged.
