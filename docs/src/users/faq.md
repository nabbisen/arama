# FAQ

## General

**Does arama send any data to the internet?**

Only for explicit AI model setup: models are downloaded from HuggingFace.
Arama never downloads ffmpeg on any platform. After resources are ready,
media processing is local. There is no telemetry or analytics.

**Which file formats are supported?**

Images: `png`, `jpg`, `jpeg`, `webp`, `gif`, `bmp`.
Videos: `mp4`.

Support for additional formats is tracked in the issue tracker.

**How much disk space does arama use?**

Around 938 MB for both AI models (CLIP 578 MB + wav2vec2 361 MB), in your
platform's standard local data directory (see
[Data locations](installation.md#data-locations)). The external
ffmpeg/ffprobe installation is outside arama's storage. The cache
database and thumbnails grow with your library — roughly 50–100 KB per
file, depending on whether embeddings have been computed.

---

## Performance

**Indexing is slow. How can I speed it up?**

- Indexing runs on the CPU. The bottleneck is CLIP inference, which
  takes roughly 0.1–0.5 seconds per image depending on hardware.
- Limit the scope with **Sub-dir depth** in Settings → General.
  Setting depth to 0 indexes only the immediate directory, which is
  much faster for exploration.
- Once a file is indexed, subsequent runs are instant (cache hit).
  Only new or changed files are re-processed.

**Video indexing is very slow.**

Video analysis requires sampling multiple frames and audio segments,
each of which runs CLIP and wav2vec2 inference. A 5-minute video may
take 30–120 seconds to index on a CPU-only machine. If you don't need
video similarity, disable **Include video** in Settings → General.

If one video cannot be decoded, sampled, or cached, arama reports an
indexing warning and continues with the remaining files. When only the
frame or audio modality succeeds for a video, arama can still use that
single modality for video similarity instead of discarding the whole
entry.

**The gallery is sluggish with many files.**

Try increasing the thumbnail size slider slightly or reducing the
subdirectory depth. Very large galleries (thousands of thumbnails) may
benefit from reducing the scope to a subdirectory at a time.

---

## Similarity results

**My similar images don't appear. What is the threshold?**

The similarity threshold defaults to **0.86** (cosine similarity). Files
scoring below this are not shown in the focus view or similarity pairs.
The threshold is configurable in **Settings → General → Similarity**
(range 0.50–1.00).

**I see false positives — unrelated images marked as similar.**

CLIP similarity captures high-level visual structure (colour palette,
scene category, composition). Photographs with similar lighting or
subject matter may score above 0.86 even when not visually "the same"
to a human eye. This is a known characteristic of CLIP-based search.

**Video similarity is not finding obvious duplicates.**

Video similarity is a weighted average of CLIP frame embeddings (60%)
and the audio track's acoustic texture (40%) — a convolutional feature
representation extracted from the wav2vec2 model's early layers, not a
full wav2vec2 embedding. If two videos have identical visuals but
different audio tracks (or vice versa), the combined score may fall
below the threshold. Re-encoded or colour-graded versions of the same
video will typically still match.

---

## Troubleshooting

**arama opens but does not start indexing my saved folder.**

If the folder saved in settings no longer exists or cannot be opened, arama
keeps that path visible, shows a startup warning, and opens the shell without
starting the cache pipeline. Choose another folder from the header to resume
indexing.

**arama shows "arama could not start" and nothing else opens.**

On every launch, arama needs to resolve or create its settings, models and
cache locations (see [Data locations](installation.md#data-locations))
before anything else runs. If any one of them cannot be created — for
example, a permissions problem on your platform's standard data directory —
arama shows this message instead of opening normally, since it has nowhere
to save that data. Check that you have write permission to your user data
directory, then restart. ffmpeg remains external on every platform
regardless.

**The app crashes on launch with a database error.**

The most common cause is a problem with arama's cache directory (see
[Data locations](installation.md#data-locations)). This directory is
created automatically; if it fails, check that you have write permission to
your platform's standard cache location.

**"ffmpeg not found" or "external ffmpeg required" after setup.**

Install a user-managed pair through a source you trust, then re-check in
**Settings → AI**. For example, on macOS:

```sh
brew install ffmpeg
```

Arama accepts a compatible `ffmpeg`/`ffprobe` pair found together on `PATH` or
in a directory selected in Settings. Legacy files left by versions before
0.40.0 in a `.arama-local/bin/` directory next to the executable are ignored
and are not deleted automatically.

**Setup downloads stall or fail.**

Check that the following domains are reachable from your network:
- `huggingface.co` (AI models)

On corporate networks, outbound HTTPS on port 443 to this domain may
need to be explicitly allowed.
