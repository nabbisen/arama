use std::path::PathBuf;
use std::sync::Arc;

use arama_ai::{
    config::video_similarity_config::VideoSimilarityConfig,
    pipeline::score::similarity::{
        image::{SimilarImagePair, find_similar_pairs, sort_pairs},
        scan_control::ScanControl,
        video::video_similarity_calculator::score_mean_vectors,
    },
};
use arama_cache::{
    CacheConfig, DbLocation, ImageCacheConfig, ImageCacheReader, LookupResult, VideoCacheConfig,
    VideoCacheReader,
};
use arama_env::{
    IMAGE_EXTENSION_ALLOWLIST, VIDEO_EXTENSION_ALLOWLIST, cache_storage_path,
    cache_thumbnail_dir_path,
};
use arama_sidecar::media::video::video_engine::FfmpegToolchain;
use iced::Task;
use rayon::prelude::*;
use swdir::DirNode;

pub mod message;
mod types;
mod update;
mod view;

use types::SimilarPair;

// Review 162: re-exported (not just `pub` inside a private `types` module)
// so `app`'s own tests can construct a payload for `Message::EmbeddingsReady`
// without widening anything else - `types` stays private, only this one
// type, already shaped for exactly this (public fields, a hand-written
// `Default`), becomes nameable from outside this crate.
pub use crate::dialog::similar_pairs_dialog::types::EmbeddingsReadyResult;
use crate::dialog::similar_pairs_dialog::types::SimilarPairItem;
use crate::dialog::similarity_read_outcome::SimilarityReadOutcome;

/// RFC 047 S3.3: one cap applied to the merged image-and-video list, not
/// one cap per media type. Images alone used to be capped here at 50
/// while videos had no cap at all - an asymmetry nobody chose, just what
/// happens when a bound is added to one path and not the other.
const MAX_SIMILAR_PAIRS: usize = 50;

#[derive(Clone, Debug)]
pub struct SimilarPairsDialog {
    dir_node: DirNode,
    pairs: Option<Vec<SimilarPair>>,
    /// How many pairs matched the threshold before `MAX_SIMILAR_PAIRS`
    /// was applied (RFC 047 S3.3). Equal to `pairs.len()` whenever
    /// nothing was truncated; the dialog only mentions a cap when the
    /// two differ, so a reader is never told "showing 50 of 50".
    total_found: usize,
    /// Set when any cache read failed while preparing `pairs` (RFC 035).
    /// One aggregated flag per dialog open, not one per failed file.
    has_read_error: bool,
    /// RFC 036: distinguishes "nothing indexed yet" from "searched and
    /// found nothing" when `pairs` is empty and there was no read error.
    nothing_indexed: bool,
    /// RFC 036: true when the directory has video paths but no
    /// ffmpeg/ffprobe pair was found, so video comparison did not run.
    ffmpeg_missing_with_videos: bool,
    hovered_media_item_path_str: Option<String>,
    similarity_threshold: f32,
    ffmpeg_toolchain: Option<FfmpegToolchain>,
    /// RFC 047 S6 / review 161 #3: `(compared, total)` as of the last
    /// `Message::Progress` tick. `(0, 0)` before scoring starts (the
    /// cache-read phase has nothing to report yet) and while `pairs` is
    /// already `Some` (nothing left to report).
    scan_progress: (usize, usize),
}

#[derive(Clone)]
struct VideoEmbedding {
    path: String,
    thumbnail_path: Option<String>,
    clip_vector: Option<Vec<f32>>,
    audio_vector: Option<Vec<f32>>,
}

impl SimilarPairsDialog {
    pub fn new<T: Into<DirNode>>(
        dir_node: T,
        pairs: Option<Vec<SimilarPair>>,
        similarity_threshold: f32,
        ffmpeg_toolchain: Option<FfmpegToolchain>,
    ) -> Self {
        let total_found = pairs.as_ref().map_or(0, Vec::len);
        Self {
            dir_node: dir_node.into(),
            pairs,
            total_found,
            has_read_error: false,
            nothing_indexed: false,
            ffmpeg_missing_with_videos: false,
            hovered_media_item_path_str: None,
            similarity_threshold,
            ffmpeg_toolchain,
            scan_progress: (0, 0),
        }
    }

    /// Current `(compared, total)` - for the caller's subscription to
    /// read against the live `ScanControl` it is holding; this method
    /// has nothing to do with that polling itself, it is just a typed
    /// accessor for `view()`/`update()`'s own stored copy.
    pub fn scan_progress(&self) -> (usize, usize) {
        self.scan_progress
    }

    /// `true` until `Message::EmbeddingsReady` has set `pairs`. Review
    /// 162's test needs this from `app` (a separate crate) to assert a
    /// stale, dropped result left the dialog untouched - everything else
    /// about `pairs` stays private.
    pub fn is_scanning(&self) -> bool {
        self.pairs.is_none()
    }

    /// RFC 047 S3.3/S6, review 161 #2/#3: the scan must be interruptible
    /// *and* report a determinate count, not only that it is running.
    ///
    /// **Why `Task::abortable()` cannot provide the cancellation**:
    /// `prepare_embeddings` is synchronous, CPU-heavy (O(N^2)) work with
    /// no await point of its own, run inside `tokio::task::spawn_blocking`
    /// so this `Task` has an await point at all - but aborting a
    /// `spawn_blocking` task has **no effect on the blocking closure
    /// itself** (`tokio-1.54.1/src/task/blocking.rs:106-110`: *"If you
    /// call abort on a spawn_blocking task, then this will not have any
    /// effect, and the task will continue running normally."*). So
    /// cancellation here uses the same mechanism as
    /// `run_thumbnail_phase`'s chunk loop: a flag the work itself checks
    /// from the inside, not a `Task`-level drop. `ScanControl` carries
    /// that flag and the progress counters `find_similar_pairs` and
    /// `find_similar_video_pairs` advance once per outer row.
    ///
    /// Returns the `Arc<ScanControl>` alongside the `Task` because this
    /// method only has `&self`, not a place on `App` to store it - the
    /// caller (the dialog-open handler) keeps it to cancel on dialog
    /// close and to poll for progress.
    ///
    /// `generation` (review 162) is the caller's own scan counter,
    /// incremented and captured at the moment this scan starts - carried
    /// through unchanged to the result's `Message::EmbeddingsReady(generation,
    /// _)`, so the caller can tell a result produced by *this* call apart
    /// from one left over by an earlier, unabortable scan that is still
    /// running when this one starts.
    pub fn default_task(&self, generation: u64) -> (Task<message::Message>, Arc<ScanControl>) {
        let dir_node = self.dir_node.clone();
        let threshold = self.similarity_threshold;
        let ffmpeg_toolchain = self.ffmpeg_toolchain.clone();
        let control = Arc::new(ScanControl::new());
        let task_control = control.clone();
        let task = Task::perform(
            async move {
                let control = task_control;
                tokio::task::spawn_blocking(move || {
                    prepare_embeddings(dir_node, threshold, ffmpeg_toolchain, &control)
                })
                .await
                .unwrap_or_default()
            },
            move |result| message::Message::EmbeddingsReady(generation, result),
        );
        (task, control)
    }
}

/// Synchronous, deliberately: every call inside it (cache reads, the
/// O(N^2) scorers) is synchronous already, and `default_task` is what
/// gives this a `spawn_blocking`/await point for cancellation - adding
/// an `async fn` here with no `.await` inside it would just recreate the
/// shape RFC 047 is about (see `handle_cache_require`'s own history).
fn prepare_embeddings(
    dir_node: DirNode,
    similarity_threshold: f32,
    ffmpeg_toolchain: Option<FfmpegToolchain>,
    control: &ScanControl,
) -> EmbeddingsReadyResult {
    let paths = dir_node.flatten_paths();

    // RFC 035: every cache-read failure below sets `had_errors` rather than
    // discarding what was already collected. An empty cache is not a
    // failure — this stays `false` unless a read actually fails.
    let mut had_errors = false;

    let db_location = match cache_storage_path() {
        Ok(path) => DbLocation::Custom(path),
        Err(err) => {
            eprintln!("failed to get cache storage path: {err}");
            return EmbeddingsReadyResult {
                outcome: SimilarityReadOutcome {
                    items: vec![],
                    had_errors: true,
                    nothing_indexed: false,
                    ffmpeg_missing_with_videos: false,
                },
                total_found: 0,
            };
        }
    };
    let read_conns = 4;
    let thumbnail_dir = match cache_thumbnail_dir_path() {
        Ok(path) => Some(path),
        Err(err) => {
            eprintln!("failed to get cache thumbnail dir path: {err}");
            return EmbeddingsReadyResult {
                outcome: SimilarityReadOutcome {
                    items: vec![],
                    had_errors: true,
                    nothing_indexed: false,
                    ffmpeg_missing_with_videos: false,
                },
                total_found: 0,
            };
        }
    };
    let cache_config = CacheConfig {
        db_location,
        read_conns,
        thumbnail_dir,
    };

    let mut image_path_embeddings: Vec<(String, Option<String>, Vec<f32>)> = vec![];
    let image_paths: Vec<&PathBuf> = paths
        .iter()
        .filter(|x| {
            x.extension().is_some_and(|x| {
                IMAGE_EXTENSION_ALLOWLIST.contains(&x.to_string_lossy().to_string().as_str())
            })
        })
        .collect();
    if !image_paths.is_empty() {
        let image_cache_reader = ImageCacheReader::as_session(ImageCacheConfig {
            cache_config: cache_config.clone(),
        });
        match image_cache_reader {
            Ok(image_cache_reader) => {
                for path in &image_paths {
                    let lookup = match image_cache_reader.lookup(path) {
                        Ok(lookup) => lookup,
                        Err(err) => {
                            eprintln!("failed to lookup image cache entry: {err}");
                            had_errors = true;
                            continue;
                        }
                    };
                    let feature = match lookup {
                        LookupResult::Hit(x) => x
                            .features
                            .map(|f| (x.path, x.thumbnail_path, f.clip_vector)),
                        _ => None,
                    };

                    if let Some(feature) = feature {
                        image_path_embeddings.push(feature);
                    }
                }
            }
            Err(err) => {
                eprintln!("failed to get image cache reader: {err}");
                had_errors = true;
            }
        }
    }

    let mut video_path_embeddings: Vec<VideoEmbedding> = vec![];
    let video_paths: Vec<&PathBuf> = paths
        .iter()
        .filter(|x| {
            x.extension().is_some_and(|x| {
                VIDEO_EXTENSION_ALLOWLIST.contains(&x.to_string_lossy().to_string().as_str())
            })
        })
        .collect();
    // RFC 036: captured before `ffmpeg_toolchain` is moved into the match
    // below. `is_none()` borrows, so this doesn't consume it early.
    let ffmpeg_missing_with_videos = !video_paths.is_empty() && ffmpeg_toolchain.is_none();
    if !video_paths.is_empty() {
        match ffmpeg_toolchain {
            Some(toolchain) => {
                let video_cache_reader = VideoCacheReader::as_session(VideoCacheConfig {
                    cache_config,
                    ffmpeg_path: Some(toolchain.ffmpeg_path().to_path_buf()),
                });
                match video_cache_reader {
                    Ok(video_cache_reader) => {
                        for path in &video_paths {
                            let lookup = match video_cache_reader.lookup(path) {
                                Ok(lookup) => lookup,
                                Err(err) => {
                                    eprintln!("failed to lookup video cache entry: {err}");
                                    had_errors = true;
                                    continue;
                                }
                            };
                            let feature = match lookup {
                                LookupResult::Hit(x) => x.features.map(|f| VideoEmbedding {
                                    path: x.path,
                                    thumbnail_path: x.thumbnail_path,
                                    clip_vector: f.clip_vector,
                                    audio_vector: f.wav2vec2_vector,
                                }),
                                _ => None,
                            };

                            if let Some(feature) = feature {
                                video_path_embeddings.push(feature);
                            }
                        }
                    }
                    Err(err) => {
                        eprintln!("failed to get video cache reader: {err}");
                        had_errors = true;
                    }
                }
            }
            None => {
                eprintln!("failed to discover a compatible ffmpeg/ffprobe pair");
                // Deliberately excluded from `had_errors` (RFC 035 §3.1):
                // video comparison never ran, so nothing failed to be read.
                // Missing ffmpeg has its own dedicated, actionable surface
                // (Settings -> AI's `settings.ai.ffmpeg_*` states) rather
                // than this dialog's generic read-failure message.
            }
        }
    }

    // RFC 036: distinguishes "nothing indexed yet" from "searched and
    // found nothing" when the final `items` list is empty. Computed from
    // what was actually collected, independent of *why* video embeddings
    // may be empty (skipped for missing ffmpeg, or genuinely unindexed) -
    // `ffmpeg_missing_with_videos` carries that distinction separately.
    let nothing_indexed = image_path_embeddings.is_empty() && video_path_embeddings.is_empty();

    // RFC 047 S6 / review 161 #3: the total is known once the cache
    // reads above finish, before scoring starts - set it here so a
    // progress tick during the very first row already has a real
    // denominator rather than 0.
    control.set_total(image_path_embeddings.len() + video_path_embeddings.len());

    // RFC 047 S3.3: images and videos are scored separately (different
    // feature spaces) but share one cap, applied once on the merged,
    // globally re-sorted list - never one cap per media type. Both
    // `find_similar_pairs` and `find_similar_video_pairs` are already
    // uncapped and individually sorted; merging two individually-sorted
    // lists still needs a full re-sort, since the merge is by
    // concatenation, not by score. Both also share `control`: its
    // `compared` count advances across images first, then videos, so
    // the dialog can report one running "N of M" across the whole scan
    // rather than two.
    let mut similar_pairs =
        find_similar_pairs(&image_path_embeddings, similarity_threshold, control);
    let video_pairs =
        find_similar_video_pairs(&video_path_embeddings, similarity_threshold, control);
    similar_pairs.extend(video_pairs);
    sort_pairs(&mut similar_pairs);
    let total_found = similar_pairs.len();
    similar_pairs.truncate(MAX_SIMILAR_PAIRS);

    let items = similar_pairs
        .into_iter()
        .map(
            |((left_path, left_thumbnail_path), (right_path, right_thumbnail_path), similarity)| {
                SimilarPair {
                    left: SimilarPairItem {
                        path: left_path,
                        thumbnail_path: left_thumbnail_path,
                    },
                    right: SimilarPairItem {
                        path: right_path,
                        thumbnail_path: right_thumbnail_path,
                    },
                    similarity,
                }
            },
        )
        .collect();
    EmbeddingsReadyResult {
        outcome: SimilarityReadOutcome {
            items,
            had_errors,
            nothing_indexed,
            ffmpeg_missing_with_videos,
        },
        total_found,
    }
}

/// Same cancellation/progress contract as `find_similar_pairs`
/// (review 161 #2/#3): `control` checked once per outer row, before its
/// O(n) inner loop, and advanced once per row actually scored. Continues
/// `control`'s running count rather than starting its own, so a scan
/// spanning images then videos reports one coherent "N of M".
fn find_similar_video_pairs(
    map: &[VideoEmbedding],
    threshold: f32,
    control: &ScanControl,
) -> Vec<SimilarImagePair> {
    let config = VideoSimilarityConfig::default();
    let mut pairs = (0..map.len())
        .into_par_iter()
        .flat_map(|i| {
            if control.is_cancelled() {
                return Vec::new();
            }
            let left = &map[i];
            let mut pairs = Vec::new();
            for right in map.iter().skip(i + 1) {
                let Some(score) = score_mean_vectors(
                    left.clip_vector.as_deref(),
                    left.audio_vector.as_deref(),
                    right.clip_vector.as_deref(),
                    right.audio_vector.as_deref(),
                    config.image_weight,
                    config.audio_weight,
                ) else {
                    continue;
                };
                if threshold <= score {
                    pairs.push((
                        (left.path.clone(), left.thumbnail_path.clone()),
                        (right.path.clone(), right.thumbnail_path.clone()),
                        score,
                    ));
                }
            }
            control.advance();
            pairs
        })
        .collect::<Vec<_>>();

    pairs.par_sort_unstable_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
    pairs
}

#[cfg(test)]
mod tests {
    use super::*;

    // RFC 047 S3.3: `prepare_embeddings` is a plain synchronous `fn`
    // now (it never awaited anything even when it was `async fn` - see
    // its own doc comment), so these tests call it directly; the
    // hand-rolled no-op-waker `block_on` this module used to need is
    // gone with it.

    // `prepare_embeddings` resolves its cache location itself via
    // `cache_storage_path()`/`cache_thumbnail_dir_path()` rather than
    // taking a `CacheConfig` parameter, so unlike
    // `media_focus_dialog::similar_media`'s helpers it cannot be pointed
    // at an isolated tempdir or an intentionally-broken path from a test.
    // Both cases below construct a `DirNode` whose `flatten_paths()`
    // means neither branch reaches the real cache at all (empty paths,
    // or a video-only path with no toolchain), so they touch no disk
    // beyond resolving path strings. The whole-lookup-failure and
    // per-item-aggregation properties are exercised for the identical
    // classification logic in `similar_media`'s tests instead, plus
    // rendered evidence of a genuinely broken cache for this dialog
    // specifically (see the review package).

    #[test]
    fn prepare_embeddings_with_no_paths_is_not_an_error() {
        let dir_node = DirNode {
            path: PathBuf::from("/nonexistent"),
            sub_dirs: vec![],
            files: vec![],
        };

        let result = prepare_embeddings(dir_node, 0.0, None, &ScanControl::new());

        assert!(
            !result.outcome.had_errors,
            "no files to check is not a failure"
        );
        assert!(result.outcome.items.is_empty());
        assert!(
            result.outcome.nothing_indexed,
            "an empty directory has nothing indexed by definition"
        );
        assert!(
            !result.outcome.ffmpeg_missing_with_videos,
            "no video paths exist"
        );
        assert_eq!(
            result.total_found, 0,
            "nothing found means nothing found before the cap either"
        );
    }

    #[test]
    fn prepare_embeddings_missing_ffmpeg_with_only_video_paths_produces_no_error() {
        let dir_node = DirNode {
            path: PathBuf::from("/nonexistent"),
            sub_dirs: vec![],
            files: vec![PathBuf::from("clip.mp4")],
        };

        let result = prepare_embeddings(dir_node, 0.0, None, &ScanControl::new());

        assert!(
            !result.outcome.had_errors,
            "missing ffmpeg has its own dedicated surface, not this dialog's error message"
        );
        assert!(result.outcome.items.is_empty());
        assert!(
            result.outcome.ffmpeg_missing_with_videos,
            "RFC 036: video paths existed but no toolchain was available"
        );
        assert_eq!(result.total_found, 0);
    }

    /// RFC 047 S3.3: proves the merge-then-cap arithmetic itself, not
    /// just that a cap exists - more than `MAX_SIMILAR_PAIRS` distinct
    /// images, all identical (so every pair is a match at any
    /// threshold), must retain exactly the cap's worth while reporting
    /// the true, larger total.
    #[test]
    fn find_similar_pairs_merge_and_cap_reports_total_found_before_truncating() {
        // n images all pairwise identical -> n*(n-1)/2 qualifying pairs.
        // MAX_SIMILAR_PAIRS is 50; 15 images make C(15,2) = 105 pairs,
        // comfortably past the cap without an unwieldy fixture.
        let n = 15;
        let vector = vec![1.0_f32, 0.0];
        let map: Vec<(String, Option<String>, Vec<f32>)> = (0..n)
            .map(|i| (format!("image-{i}.jpg"), None, vector.clone()))
            .collect();

        let all_pairs = find_similar_pairs(&map, 0.0, &ScanControl::new());
        let total_found = all_pairs.len();
        let mut capped = all_pairs;
        capped.truncate(MAX_SIMILAR_PAIRS);

        assert_eq!(total_found, n * (n - 1) / 2, "105 qualifying pairs");
        assert_eq!(
            capped.len(),
            MAX_SIMILAR_PAIRS,
            "retained must be exactly the cap, not the full 105"
        );
        assert!(
            total_found > capped.len(),
            "this fixture only proves anything if the cap actually bit"
        );
    }

    /// Review 161 #2/#3, for `find_similar_video_pairs` specifically -
    /// the identical property already proven for `find_similar_pairs`
    /// in `arama-ai`'s own tests, reproduced here because this function
    /// is private to this crate and has its own cancellation check.
    /// Deterministic for the same reason: triggers on observed
    /// progress, not a sleep.
    #[test]
    fn find_similar_video_pairs_cancelling_once_progress_is_observed_stops_short() {
        let n = 400;
        let map: Vec<VideoEmbedding> = (0..n)
            .map(|i| VideoEmbedding {
                path: format!("clip-{i}.mp4"),
                thumbnail_path: None,
                clip_vector: Some(vec![1.0, 0.0]),
                audio_vector: None,
            })
            .collect();
        let control = Arc::new(ScanControl::new());

        let canceller = {
            let control = control.clone();
            std::thread::spawn(move || {
                while control.snapshot().0 == 0 {
                    std::hint::spin_loop();
                }
                control.cancel();
            })
        };

        let _pairs = find_similar_video_pairs(&map, 0.0, &control);
        canceller.join().unwrap();

        let (compared, _) = control.snapshot();
        assert!(
            compared < n,
            "expected the scan to stop before every row was compared, got all {n}"
        );
        assert!(
            compared > 0,
            "expected at least the triggering row to count"
        );
    }

    /// Review 161 #3: a scan with both images and videos must report one
    /// running total across both, not reset between them - confirmed by
    /// calling `prepare_embeddings`'s own merge sequence through a
    /// `ScanControl` and checking its `total` after cache-read-equivalent
    /// setup, without needing a real cache (this only exercises the
    /// scoring half, by constructing the embeddings directly).
    #[test]
    fn total_spans_images_and_videos_together() {
        let control = ScanControl::new();
        let images: Vec<(String, Option<String>, Vec<f32>)> = (0..3)
            .map(|i| (format!("image-{i}.jpg"), None, vec![1.0, 0.0]))
            .collect();
        let videos: Vec<VideoEmbedding> = (0..2)
            .map(|i| VideoEmbedding {
                path: format!("clip-{i}.mp4"),
                thumbnail_path: None,
                clip_vector: Some(vec![1.0, 0.0]),
                audio_vector: None,
            })
            .collect();

        control.set_total(images.len() + videos.len());
        let _ = find_similar_pairs(&images, 0.0, &control);
        let _ = find_similar_video_pairs(&videos, 0.0, &control);

        let (compared, total) = control.snapshot();
        assert_eq!(total, 5, "3 images + 2 videos");
        assert_eq!(
            compared, 5,
            "every row across both calls must advance the one shared counter"
        );
    }
}
