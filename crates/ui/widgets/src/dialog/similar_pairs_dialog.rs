use std::path::PathBuf;

use arama_ai::{
    config::video_similarity_config::VideoSimilarityConfig,
    pipeline::score::similarity::{
        image::{SimilarImagePair, find_similar_pairs},
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

use crate::dialog::similar_pairs_dialog::types::SimilarPairItem;
use crate::dialog::similarity_read_outcome::SimilarityReadOutcome;

const MAX_IMAGE_SIMILAR_PAIRS: usize = 50;

#[derive(Clone, Debug)]
pub struct SimilarPairsDialog {
    dir_node: DirNode,
    pairs: Option<Vec<SimilarPair>>,
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
        Self {
            dir_node: dir_node.into(),
            pairs,
            has_read_error: false,
            nothing_indexed: false,
            ffmpeg_missing_with_videos: false,
            hovered_media_item_path_str: None,
            similarity_threshold,
            ffmpeg_toolchain,
        }
    }

    pub fn default_task(&self) -> Task<message::Message> {
        let dir_node = self.dir_node.clone();
        let threshold = self.similarity_threshold;
        let ffmpeg_toolchain = self.ffmpeg_toolchain.clone();
        Task::perform(
            prepare_embeddings(dir_node, threshold, ffmpeg_toolchain),
            message::Message::EmbeddingsReady,
        )
    }
}

async fn prepare_embeddings(
    dir_node: DirNode,
    similarity_threshold: f32,
    ffmpeg_toolchain: Option<FfmpegToolchain>,
) -> SimilarityReadOutcome<SimilarPair> {
    let paths = dir_node.flatten_paths();

    // RFC 035: every cache-read failure below sets `had_errors` rather than
    // discarding what was already collected. An empty cache is not a
    // failure — this stays `false` unless a read actually fails.
    let mut had_errors = false;

    let db_location = match cache_storage_path() {
        Ok(path) => DbLocation::Custom(path),
        Err(err) => {
            eprintln!("failed to get cache storage path: {err}");
            return SimilarityReadOutcome {
                items: vec![],
                had_errors: true,
                nothing_indexed: false,
                ffmpeg_missing_with_videos: false,
            };
        }
    };
    let read_conns = 4;
    let thumbnail_dir = match cache_thumbnail_dir_path() {
        Ok(path) => Some(path),
        Err(err) => {
            eprintln!("failed to get cache thumbnail dir path: {err}");
            return SimilarityReadOutcome {
                items: vec![],
                had_errors: true,
                nothing_indexed: false,
                ffmpeg_missing_with_videos: false,
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

    let mut similar_pairs = find_similar_pairs(
        &image_path_embeddings,
        similarity_threshold,
        MAX_IMAGE_SIMILAR_PAIRS,
    );
    let video_pairs = find_similar_video_pairs(&video_path_embeddings, similarity_threshold);
    similar_pairs.extend(video_pairs);
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
    SimilarityReadOutcome {
        items,
        had_errors,
        nothing_indexed,
        ffmpeg_missing_with_videos,
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

    /// `prepare_embeddings` is `async fn` but never actually awaits
    /// anything (every call inside it is synchronous); this drives it to
    /// completion without pulling in an async-runtime dependency this
    /// crate does not otherwise need.
    fn block_on<F: std::future::Future>(fut: F) -> F::Output {
        use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
        fn noop(_: *const ()) {}
        fn clone_raw(_: *const ()) -> RawWaker {
            RawWaker::new(std::ptr::null(), &VTABLE)
        }
        static VTABLE: RawWakerVTable = RawWakerVTable::new(clone_raw, noop, noop, noop);
        let raw = RawWaker::new(std::ptr::null(), &VTABLE);
        let waker = unsafe { Waker::from_raw(raw) };
        let mut cx = Context::from_waker(&waker);
        let mut fut = std::pin::pin!(fut);
        loop {
            if let Poll::Ready(val) = fut.as_mut().poll(&mut cx) {
                return val;
            }
        }
    }

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

        let outcome = block_on(prepare_embeddings(dir_node, 0.0, None));

        assert!(!outcome.had_errors, "no files to check is not a failure");
        assert!(outcome.items.is_empty());
        assert!(
            outcome.nothing_indexed,
            "an empty directory has nothing indexed by definition"
        );
        assert!(!outcome.ffmpeg_missing_with_videos, "no video paths exist");
    }

    #[test]
    fn prepare_embeddings_missing_ffmpeg_with_only_video_paths_produces_no_error() {
        let dir_node = DirNode {
            path: PathBuf::from("/nonexistent"),
            sub_dirs: vec![],
            files: vec![PathBuf::from("clip.mp4")],
        };

        let outcome = block_on(prepare_embeddings(dir_node, 0.0, None));

        assert!(
            !outcome.had_errors,
            "missing ffmpeg has its own dedicated surface, not this dialog's error message"
        );
        assert!(outcome.items.is_empty());
        assert!(
            outcome.ffmpeg_missing_with_videos,
            "RFC 036: video paths existed but no toolchain was available"
        );
    }
}
