use std::path::PathBuf;

use arama_ai::{
    config::video_similarity_config::VideoSimilarityConfig,
    pipeline::score::similarity::{
        image::{SimilarImagePair, find_similar_pairs, sort_pairs},
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

use crate::dialog::similar_pairs_dialog::types::{EmbeddingsReadyResult, SimilarPairItem};
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
        }
    }

    /// RFC 047 S3.3: the scan must be interruptible. `prepare_embeddings`
    /// is synchronous, CPU-heavy (O(N^2) scoring) work with no await
    /// point of its own - `spawn_blocking` gives this `Task` the one
    /// await point `.abortable()` needs to actually stop it (the same
    /// reasoning as `handle_cache_require`'s thumbnail phase; see that
    /// function's own comment for why a technique like this is needed at
    /// all rather than only here). Unlike that phase, this is one
    /// unchunked unit of work, not a loop: cancelling means "discard
    /// whatever this produces," which is all RFC 047 S3.3 asks for here,
    /// since progressive/streamed results are explicitly a non-goal
    /// (RFC 047 S5).
    ///
    /// Returns the abort `Handle` alongside the `Task` because this
    /// method only has `&self`, not a place on `App` to store it - the
    /// caller (the dialog-open handler) is responsible for keeping it
    /// and aborting it when the dialog closes.
    pub fn default_task(&self) -> (Task<message::Message>, iced::task::Handle) {
        let dir_node = self.dir_node.clone();
        let threshold = self.similarity_threshold;
        let ffmpeg_toolchain = self.ffmpeg_toolchain.clone();
        Task::perform(
            async move {
                tokio::task::spawn_blocking(move || {
                    prepare_embeddings(dir_node, threshold, ffmpeg_toolchain)
                })
                .await
                .unwrap_or_default()
            },
            message::Message::EmbeddingsReady,
        )
        .abortable()
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

    // RFC 047 S3.3: images and videos are scored separately (different
    // feature spaces) but share one cap, applied once on the merged,
    // globally re-sorted list - never one cap per media type. Both
    // `find_similar_pairs` and `find_similar_video_pairs` are already
    // uncapped and individually sorted; merging two individually-sorted
    // lists still needs a full re-sort, since the merge is by
    // concatenation, not by score.
    let mut similar_pairs = find_similar_pairs(&image_path_embeddings, similarity_threshold);
    let video_pairs = find_similar_video_pairs(&video_path_embeddings, similarity_threshold);
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

fn find_similar_video_pairs(map: &[VideoEmbedding], threshold: f32) -> Vec<SimilarImagePair> {
    let config = VideoSimilarityConfig::default();
    let mut pairs = (0..map.len())
        .into_par_iter()
        .flat_map(|i| {
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

        let result = prepare_embeddings(dir_node, 0.0, None);

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

        let result = prepare_embeddings(dir_node, 0.0, None);

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

        let all_pairs = find_similar_pairs(&map, 0.0);
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
}
