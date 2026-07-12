# RFC 018 — AI/video pipeline resilience

**Status.** Implemented (Unreleased)
**Tracks.** Roadmap follow-up: define how video indexing and AI embedding
workflows handle per-file, per-modality, cache, and sidecar failures without
turning one bad media file into a misleading or unnecessarily aborted indexing
run.
**Touches.** `crates/ai/src/pipeline/encode/image/embeddings.rs`,
`crates/ai/src/pipeline_manager/video_similarity_pipeline.rs`,
`crates/ai/src/pipeline/extract/video_extractor.rs`,
`crates/ai/src/config/video_similarity_config.rs`,
`app/src/core/update/cache.rs`, `crates/cache/`, `docs/src/users/faq.md`,
`docs/src/dev/architecture.md`, `docs/src/dev/workspace.md`,
`CHANGELOG.md`, `rfcs/README.md`.

## Summary

arama currently treats some video and AI pipeline failures as fatal to the whole
embedding run, while other failures are logged and skipped locally. The behavior
is not yet a deliberate policy.

This RFC proposes a first resilience pass for video indexing and AI embedding:

1. Classify failures by scope: fatal pipeline setup, per-file failure,
   per-modality setup failure, per-modality extraction failure, cache write
   failure, and developer invariant.
2. Let one unreadable or unsupported media file fail without aborting the whole
   directory indexing run.
3. Allow video feature extraction to proceed with whichever modality is valid
   when that is safe: CLIP frames, wav2vec2 audio, or both.
4. Return a structured indexing summary so the app can show a concise warning
   toast when some files failed.
5. Keep scoring semantics explicit when a video has only one modality.

This RFC does not change model selection, sampling weights, cache schema, or
similarity thresholds unless the implementation needs a small metadata marker
to avoid misleading cache hits.

## Why

Video indexing is the most fragile media workflow:

- `image_embedding()` aborts the whole embedding task when
  `VideoSimilarityPipeline::preload()` fails for one video.
- `VideoSimilarityPipeline::get_or_extract()` still calls
  `VideoEngine::ffmpeg().expect("failed to get ffmpeg command")` while creating
  the video cache writer.
- `VideoExtractor::extract_video_frames()` logs individual frame extraction
  failures and continues, but audio extraction currently collects per-segment
  errors into a fatal result.
- Empty frame/audio outputs can become empty vectors, which are useful as a
  fallback only if the downstream scoring policy treats them deliberately.
- Cache write failure currently aborts the file's pipeline path, but the app
  needs a clear distinction between "feature extraction failed" and "feature
  extracted but not persisted".

The user-visible result today can be confusing: one bad video can stop an
otherwise valid indexing run, or stderr-only partial extraction can produce
hard-to-explain cache contents.

## Design

### Part A — Failure classification

Classify AI/video indexing failures into six categories:

| Category | Meaning | First-pass behavior |
|----------|---------|---------------------|
| Fatal pipeline setup | No requested media type can do useful work because required shared setup failed | Abort run and surface existing app error toast |
| Per-file failure | One source file cannot be read, decoded, sampled, encoded, or cached enough to be useful | Skip that file and record a warning |
| Per-modality setup failure | One model or sidecar needed by one modality cannot initialize | Disable that modality, continue with other valid modalities/files |
| Per-modality failure | Video frame path or audio path fails independently | Continue with the other modality if non-empty |
| Cache write failure | Features were extracted but not persisted | Record warning; continue indexing next files |
| Developer invariant | Impossible internal state or test-only assumptions | Keep existing panic/expect only when defensible |

Setup is classified per requested capability, not only per process:

- CLIP setup failure disables image-file embedding and the video frame
  modality. It is fatal only when no requested work can proceed without CLIP.
  If wav2vec2 and ffmpeg/ffprobe are available, video files may still produce
  audio-only entries.
- wav2vec2 setup failure disables the video audio modality. It is not fatal
  when CLIP frame extraction can still produce usable video entries or image
  indexing can proceed.
- ffmpeg/ffprobe setup failure disables video extraction. It is not fatal for
  unrelated image indexing. Video files in that run are skipped with warnings.
- Cache path initialization remains fatal for cache-backed indexing because no
  current file result can be read or persisted truthfully.

Fatal setup therefore means "nothing useful can be done for the selected run,"
not "one optional modality failed."

### Part B — Structured embedding summary

Replace the `Option<String>` error return from the embedding task with a
structured summary or equivalent:

```rust
pub struct EmbeddingRunReport {
    /// Files that produced usable embeddings for this run, regardless of
    /// whether persistence later failed.
    pub processed: usize,
    pub skipped: Vec<EmbeddingFileIssue>,
    pub cache_write_failures: Vec<EmbeddingFileIssue>,
}

pub struct EmbeddingFileIssue {
    pub path: PathBuf,
    pub message: String,
}
```

Equivalent names are acceptable. The important contract is:

- task-level fatal setup errors still return `Err`;
- per-file and cache-write issues accumulate into the report;
- `processed` counts files with usable extracted or cached embeddings. If
  extraction succeeds but cache persistence fails, the file still increments
  `processed` and also appears in `cache_write_failures`;
- `skipped` counts files with no usable embedding for the current run;
- the app can show one warning toast such as "Indexed with 3 skipped files";
- detailed paths should be available for logs or future inline diagnostics
  without overwhelming the toast body.

### Part C — Video modality policy

Video extraction should distinguish frame and audio failures:

- If frame extraction yields at least one valid frame, CLIP video embeddings are
  valid.
- If audio extraction yields at least one valid segment, wav2vec2 embeddings are
  valid.
- If one modality fails but the other produces a vector, cache and use the
  partial video entry.
- If both modalities fail or produce empty vectors, skip the file and record a
  per-file failure.

Scoring must make partial entries explicit. First-pass comparison matrix:

| Left entry | Right entry | Score rule |
|------------|-------------|------------|
| image+audio | image+audio | configured weighted score: `image_weight * image + audio_weight * audio` |
| image-only | image-only | image similarity only |
| image-only | image+audio | image similarity only |
| image+audio | image-only | image similarity only |
| audio-only | audio-only | audio similarity only |
| audio-only | image+audio | audio similarity only |
| image+audio | audio-only | audio similarity only |
| image-only | audio-only | invalid pair; do not return a similarity result |
| any | neither | invalid entry; do not cache or compare |

This avoids treating a missing modality as a zero score that unfairly drags down
otherwise useful similarity. Single-modality comparisons use the same threshold
as normal video similarity; weights are not renormalized because only the
available shared modality is scored. If the current cache payload cannot
distinguish "missing because not computed" from "computed empty", the
implementation should add the smallest local marker needed or compute validity
from non-empty vectors and document the limitation.

### Part D — Sidecar and cache behavior

Remove the ffmpeg command `expect()` in the video cache writer path. Sidecar
resolution failures should become per-file/video-scope failures unless they
happen before any file work and prevent all requested work.

Cache writes should not abort the whole directory run. If feature extraction
succeeds but `VideoCacheWriter::upsert()` or `ImageCacheWriter::upsert()` fails,
record a cache-write failure and continue. The next run may recompute that file.

### Part E — User-visible reporting

Use RFC 017's recoverable error policy:

- fatal setup failure remains an app-level error toast;
- partial indexing completion becomes a warning toast with counts;
- no per-file modal or blocking inline surface in the first pass.

Example warning copy:

> Indexed with warnings: 2 files skipped, 1 cache write failed.

The toast should stay concise. Detailed per-file diagnostics can remain in logs
or be saved for a later diagnostics view.

## Touches in detail

### `crates/ai/src/pipeline/encode/image/embeddings.rs`

Change the top-level embedding loop to accumulate per-file issues instead of
returning on the first file-level failure. Preserve fatal setup errors for model
initialization and cache path initialization.

### `crates/ai/src/pipeline_manager/video_similarity_pipeline.rs`

Return structured video extraction/cache outcomes. Remove the ffmpeg command
`expect()` and classify sidecar failures according to this RFC.

### `crates/ai/src/pipeline/extract/video_extractor.rs`

Separate frame and audio extraction outcomes. Avoid stderr-only partial errors
where callers need structured decisions.

### `crates/ai/src/config/video_similarity_config.rs`

Keep sampling and weights unchanged unless tests reveal a required explicit
partial-modality score helper.

### `app/src/core/update/cache.rs`

Map the embedding report into success/warning/error toasts. The current
directory reload behavior should remain unchanged.

### `crates/cache/`

Avoid schema churn if possible. If partial modality validity needs an explicit
marker, keep it local and versioned through the existing payload version
mechanism.

### `docs/src/users/faq.md`

Document that some video files may be skipped when decoding or audio/frame
extraction fails, while the rest of the indexing run can continue.

### `docs/src/dev/architecture.md` and `docs/src/dev/workspace.md`

Document the per-file/per-modality resilience policy.

### `CHANGELOG.md`

Record the behavior change under `[Unreleased]`.

### `rfcs/README.md`

List RFC 018 in the Implemented table once shipped.

## Non-goals

- No scoring algorithm redesign beyond explicit partial-modality handling.
- No model replacement or new AI dependency.
- No background retry queue.
- No persistent diagnostics UI.
- No release action. Release timing remains owner-driven.
- No cache schema migration unless partial-modality correctness requires it.

## Risks

- Partial video entries can make similarity semantics harder to explain.
  Mitigation: compare only over modalities present in both entries, using the
  matrix in Part C.
- Continuing after cache write failure can recompute expensive work on the next
  run. Mitigation: warn the user and keep the behavior recoverable.
- Warning toasts with many file paths can be noisy. Mitigation: report counts
  in the toast and keep detailed paths out of the first-pass UI.
- Treating ffmpeg absence as per-video skip could hide setup problems.
  Mitigation: if the selected run contains only videos and ffmpeg is missing,
  the warning count should make the skipped scope obvious.

## Test plan

- Unit tests for report aggregation:
  - one image/video file failure does not abort later files;
  - cache write failure is counted separately from extraction failure.
- Unit tests for partial video modality policy:
  - image-only vectors are valid;
  - audio-only vectors are valid;
  - both-empty vectors skip the file.
- Regression test or focused helper test proving ffmpeg command resolution no
  longer panics in the video writer path.
- Existing scoring tests remain green.
- Workspace gates:
  - `cargo fmt --all --check`
  - `cargo check --workspace`
  - `cargo test -p arama-ai`
  - `cargo test --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`

## Open questions

1. Should per-file issue paths appear in the toast body, or stay out of the UI
   until a diagnostics surface exists?
2. If cache payload markers are needed, should this become a payload-version
   bump in the first implementation or a follow-up RFC?
