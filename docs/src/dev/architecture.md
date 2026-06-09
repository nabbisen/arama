# Architecture Overview

## System diagram

```
┌──────────────────────────────────────────────────────────────┐
│  arama (desktop app — iced 0.14 + snora 0.8)                 │
│                                                              │
│  ┌──────────────────────────┐  ┌────────────────────────┐   │
│  │  Explorer page           │  │  Settings page         │   │
│  │  ┌──────┬──────────────┐ │  │  General / AI /        │   │
│  │  │ Dir  │  Gallery     │ │  │  File system / About   │   │
│  │  │ tree │  thumbnails  │ │  └────────────────────────┘   │
│  │  └──────┴──────────────┘ │                               │
│  └──────────────────────────┘                               │
│            │                                                 │
│  ┌─────────▼──────────────────────────────────────────────┐ │
│  │  AI pipeline  (arama-ai)                               │ │
│  │                                                        │ │
│  │  images ──► CLIP encoder ──────────────────► vec[512]  │ │
│  │                                                        │ │
│  │  videos ──► frame sampler ──► CLIP encoder ► vec[512]  │ │
│  │         └──► audio sampler ──► wav2vec2 ──► vec[768]   │ │
│  │                                                        │ │
│  │  similarity = dot(normalize(a), normalize(b))          │ │
│  └────────────────────────┬───────────────────────────────┘ │
│                           │                                 │
│  ┌────────────────────────▼───────────────────────────────┐ │
│  │  arama-cache  (localcache, SQLite)                     │ │
│  │  namespace "image" — thumbnail path, CLIP vector       │ │
│  │  namespace "video" — thumbnail path, CLIP + wav2vec2   │ │
│  └────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────┘
```

## Message and data flow

arama follows the iced Elm-like architecture. All state lives in `App`;
every interaction produces a `Message` that flows through `App::update`.
The AI pipeline and cache writes are spawned as `Task::perform` futures
so they do not block the UI event loop.

### Indexing flow

```
User selects directory
        │
        ▼
App::update / on_dir_changed
  ├── abort any running task (Task::abortable handle)
  └── Task::done(CacheRequire)
               │
               ▼
        CacheRequire handler
          └── Task::perform(upsert_all thumbnails, ThumbnailCacheFinished)
                              │
                              ▼
                    ThumbnailCacheFinished
                      ├── update gallery thumbnail map
                      └── Task::perform(image_embedding, EmbeddingCacheFinished)
                                         │
                                         ▼
                               EmbeddingCacheFinished
                                 └── processing_off()
```

### Similarity search flow

```
User clicks a thumbnail
        │
        ▼
App opens MediaFocusDialog
        │
        ▼
similar_images() / similar_videos()
  ├── ImageCacheReader::all_in_dir() or all()
  ├── partition: target entry vs candidates
  ├── rayon::par_iter: compute dot(target, candidate) for each
  └── filter by threshold → sort descending → return Vec<SimilarMediaItem>
```

## AI models

| Model | Purpose | Source |
|---|---|---|
| `clip-vit-base-patch32` | 512-dim image/frame embeddings | HuggingFace `openai/` |
| `wav2vec2-base-960h` | 768-dim audio embeddings | HuggingFace `facebook/` |

Both models run on CPU via [candle](https://github.com/huggingface/candle).

## Video sampling strategy

For each video, `VideoSimilarityConfig` computes a set of timestamps:

- **Head zone** — fixed anchors at 3, 9, 15 seconds + 5 evenly-spaced
  points in the first 135 seconds. Gives dense coverage of the opening.
- **Middle** — 3 percentage anchors at 30%, 50%, 70% of duration.
- **Tail** — fixed anchors at 30, 15, 5 seconds before the end.

Nearby timestamps (within 20 seconds) are merged. The result is
typically 14 sample points per video.

The final video similarity score is:
```
score = 0.60 * clip_similarity + 0.40 * wav2vec2_similarity
```

## Cache design

Two SQLite namespaces in one file (`.arama-cache/cache-v2.sqlite`):

| Namespace | Key | Payload |
|---|---|---|
| `image` | canonicalized file path | `thumbnail_path`, `clip_vector` |
| `video` | canonicalized file path | `thumbnail_path`, `clip_vector`, `wav2vec2_vector` |

Change detection uses `MetadataThenFullHash` (BLAKE3): metadata
(mtime + size) is checked first; hash is only computed when metadata
differs. As of localcache v0.20, mtime is stored at nanosecond
precision.

Thumbnail files are named `blake3(canonical_path)[..16].jpg` in
`.arama-cache/thumbnail/`.
