use std::path::Path;

use arama_ai::{
    config::video_similarity_config::VideoSimilarityConfig,
    pipeline::score::similarity::video::video_similarity_calculator::score_mean_vectors,
};
use arama_sidecar::media::video::video_engine::FfmpegToolchain;
use rayon::prelude::*;

use arama_cache::{
    CacheConfig, DbLocation, ImageCacheConfig, ImageCacheReader, VideoCacheConfig, VideoCacheReader,
};
use arama_env::{
    VIDEO_EXTENSION_ALLOWLIST, cache_lookup_strategy::CacheLookupStrategy, cache_storage_path,
    cache_thumbnail_dir_path,
};

use crate::dialog::similarity_read_outcome::SimilarityReadOutcome;

use super::{MediaFocusDialog, types::SimilarMediaItem};

impl MediaFocusDialog {
    pub fn similar_media(&self) -> SimilarityReadOutcome<SimilarMediaItem> {
        let threshold = self.similarity_threshold;
        let path = &self.history[self.history_index];

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

        let is_video = path.extension().is_some_and(|x| {
            VIDEO_EXTENSION_ALLOWLIST.contains(&x.to_string_lossy().to_string().as_str())
        });

        if is_video {
            similar_videos(
                path,
                cache_config,
                self.cache_lookup_strategy,
                threshold,
                self.ffmpeg_toolchain.as_ref(),
            )
        } else {
            similar_images(path, cache_config, self.cache_lookup_strategy, threshold)
        }
    }
}

fn similar_images(
    path: &Path,
    cache_config: CacheConfig,
    cache_lookup_strategy: CacheLookupStrategy,
    threshold: f32,
) -> SimilarityReadOutcome<SimilarMediaItem> {
    let image_cache_reader = match ImageCacheReader::as_session(ImageCacheConfig { cache_config }) {
        Ok(reader) => reader,
        Err(err) => {
            eprintln!("failed to get image cache reader: {err}");
            return SimilarityReadOutcome {
                items: vec![],
                had_errors: true,
                nothing_indexed: false,
                ffmpeg_missing_with_videos: false,
            };
        }
    };

    let cache_lookuped = match cache_lookup_strategy {
        CacheLookupStrategy::Everywhere => image_cache_reader.all(),
        CacheLookupStrategy::CurrentDirAndSubDirs => {
            image_cache_reader.all_in_dir_and_sub_dirs(path)
        }
        CacheLookupStrategy::CurrentDirOnly => image_cache_reader.all_in_dir(path),
    };

    let mut had_batch_errors = false;
    let cache_entries = match cache_lookuped {
        // `.flatten()` alone would silently drop each per-entry `Err` here
        // — the same per-item-continue shape RFC 035 exists to close, just
        // hidden inside a batch call instead of an explicit loop.
        Ok(entries) => entries
            .into_iter()
            .filter_map(|entry| match entry {
                Ok(entry) => Some(entry),
                Err(err) => {
                    eprintln!("failed to lookup image cache entry: {err}");
                    had_batch_errors = true;
                    None
                }
            })
            .collect::<Vec<_>>(),
        Err(err) => {
            eprintln!("failed to lookup image cache entries: {err}");
            return SimilarityReadOutcome {
                items: vec![],
                had_errors: true,
                nothing_indexed: false,
                ffmpeg_missing_with_videos: false,
            };
        }
    };

    // Split target and candidate entries.
    let canonical_path = match path.canonicalize() {
        Ok(path) => path.to_string_lossy().to_string(),
        Err(err) => {
            // In scope (RFC 035 §3.1): not a CacheError, but a real
            // failure that would otherwise return an empty vector
            // indistinguishable from "no similar images."
            eprintln!("failed to canonicalize image path: {err}");
            return SimilarityReadOutcome {
                items: vec![],
                had_errors: true,
                nothing_indexed: false,
                ffmpeg_missing_with_videos: false,
            };
        }
    };
    let (target_item, candidates): (Vec<_>, Vec<_>) = cache_entries
        .into_iter()
        .partition(|x| x.path == canonical_path);

    let Some(target_clip_vector) = target_item
        .first()
        .and_then(|target| target.features.as_ref())
        .map(|features| features.clip_vector.to_owned())
    else {
        // Out of scope (RFC 035 §3.1): the target item simply is not
        // indexed yet. An ordinary empty state, not a failure — unless a
        // batch read error already happened, in which case the target's
        // own entry may be exactly what silently failed above; that must
        // not be reported as an ordinary absence.
        return SimilarityReadOutcome {
            items: vec![],
            had_errors: had_batch_errors,
            // RFC 036: the target genuinely isn't indexed - this holds
            // regardless of `had_batch_errors`, since the view suppresses
            // this message whenever `had_errors` is set anyway.
            nothing_indexed: true,
            ffmpeg_missing_with_videos: false,
        };
    };

    // Compute similarities in parallel.
    let mut ret = candidates
        .into_par_iter()
        .filter_map(|x| {
            let features = x.features?;
            let similarity = dot_product(&target_clip_vector, &features.clip_vector);
            Some(SimilarMediaItem {
                path: x.path,
                thumbnail_path: x.thumbnail_path,
                similarity,
            })
        })
        .filter(|x| threshold <= x.similarity)
        .collect::<Vec<_>>();

    // Sort by descending similarity. Unstable sort is faster and ordering of
    // exact ties is irrelevant here.
    ret.sort_unstable_by(|a, b| {
        b.similarity
            .partial_cmp(&a.similarity)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    SimilarityReadOutcome {
        items: ret,
        had_errors: had_batch_errors,
        // The target was indexed (we reached here), so an empty `ret` is
        // "searched and found nothing", not "nothing indexed yet".
        nothing_indexed: false,
        ffmpeg_missing_with_videos: false,
    }
}

fn similar_videos(
    path: &Path,
    cache_config: CacheConfig,
    cache_lookup_strategy: CacheLookupStrategy,
    threshold: f32,
    toolchain: Option<&FfmpegToolchain>,
) -> SimilarityReadOutcome<SimilarMediaItem> {
    let ffmpeg_path = match toolchain {
        Some(toolchain) => toolchain.ffmpeg_path().to_path_buf(),
        None => {
            eprintln!("failed to discover a compatible ffmpeg/ffprobe pair");
            // Deliberately excluded from `had_errors` (RFC 035 §3.1): video
            // comparison never ran, so nothing failed to be read. Missing
            // ffmpeg has its own dedicated surface in Settings -> AI.
            // RFC 036: `ffmpeg_missing_with_videos` carries this instead,
            // so the dialog still says something rather than nothing.
            return SimilarityReadOutcome {
                items: vec![],
                had_errors: false,
                nothing_indexed: false,
                ffmpeg_missing_with_videos: true,
            };
        }
    };
    let video_cache_reader = match VideoCacheReader::as_session(VideoCacheConfig {
        cache_config,
        ffmpeg_path: Some(ffmpeg_path),
    }) {
        Ok(reader) => reader,
        Err(err) => {
            eprintln!("failed to get video cache reader: {err}");
            return SimilarityReadOutcome {
                items: vec![],
                had_errors: true,
                nothing_indexed: false,
                ffmpeg_missing_with_videos: false,
            };
        }
    };

    let cache_lookuped = match cache_lookup_strategy {
        CacheLookupStrategy::Everywhere => video_cache_reader.all(),
        CacheLookupStrategy::CurrentDirAndSubDirs => {
            video_cache_reader.all_in_dir_and_sub_dirs(path)
        }
        CacheLookupStrategy::CurrentDirOnly => video_cache_reader.all_in_dir(path),
    };

    let mut had_batch_errors = false;
    let cache_entries = match cache_lookuped {
        // `.flatten()` alone would silently drop each per-entry `Err` here
        // — the same per-item-continue shape RFC 035 exists to close, just
        // hidden inside a batch call instead of an explicit loop.
        Ok(entries) => entries
            .into_iter()
            .filter_map(|entry| match entry {
                Ok(entry) => Some(entry),
                Err(err) => {
                    eprintln!("failed to lookup video cache entry: {err}");
                    had_batch_errors = true;
                    None
                }
            })
            .collect::<Vec<_>>(),
        Err(err) => {
            eprintln!("failed to lookup video cache entries: {err}");
            return SimilarityReadOutcome {
                items: vec![],
                had_errors: true,
                nothing_indexed: false,
                ffmpeg_missing_with_videos: false,
            };
        }
    };

    // Split target and candidate entries.
    let canonical_path = match path.canonicalize() {
        Ok(path) => path.to_string_lossy().to_string(),
        Err(err) => {
            // In scope (RFC 035 §3.1): not a CacheError, but a real
            // failure that would otherwise return an empty vector
            // indistinguishable from "no similar videos."
            eprintln!("failed to canonicalize video path: {err}");
            return SimilarityReadOutcome {
                items: vec![],
                had_errors: true,
                nothing_indexed: false,
                ffmpeg_missing_with_videos: false,
            };
        }
    };
    let (target_item, candidates): (Vec<_>, Vec<_>) = cache_entries
        .into_iter()
        .partition(|x| x.path == canonical_path);

    let Some(target_features) = target_item
        .first()
        .and_then(|target| target.features.as_ref())
    else {
        // Out of scope (RFC 035 §3.1): the target item simply is not
        // indexed yet. An ordinary empty state, not a failure — unless a
        // batch read error already happened, in which case the target's
        // own entry may be exactly what silently failed above; that must
        // not be reported as an ordinary absence.
        return SimilarityReadOutcome {
            items: vec![],
            had_errors: had_batch_errors,
            // RFC 036: the target genuinely isn't indexed - this holds
            // regardless of `had_batch_errors`, since the view suppresses
            // this message whenever `had_errors` is set anyway.
            nothing_indexed: true,
            ffmpeg_missing_with_videos: false,
        };
    };
    let video_similarity_config = VideoSimilarityConfig::default();

    // Compute similarities in parallel.
    let mut ret = candidates
        .into_par_iter()
        .filter_map(|x| {
            let similarity = match &x.features {
                Some(features) => score_mean_vectors(
                    target_features.clip_vector.as_deref(),
                    target_features.wav2vec2_vector.as_deref(),
                    features.clip_vector.as_deref(),
                    features.wav2vec2_vector.as_deref(),
                    video_similarity_config.image_weight,
                    video_similarity_config.audio_weight,
                )?,
                _ => return None,
            };

            Some(SimilarMediaItem {
                path: x.path,
                thumbnail_path: x.thumbnail_path,
                similarity,
            })
        })
        .filter(|x| threshold <= x.similarity)
        .collect::<Vec<_>>();

    // Sort by descending similarity. Unstable sort is faster and ordering of
    // exact ties is irrelevant here.
    ret.sort_unstable_by(|a, b| {
        b.similarity
            .partial_cmp(&a.similarity)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    SimilarityReadOutcome {
        items: ret,
        had_errors: had_batch_errors,
        // The target was indexed (we reached here), so an empty `ret` is
        // "searched and found nothing", not "nothing indexed yet".
        nothing_indexed: false,
        ffmpeg_missing_with_videos: false,
    }
}

fn dot_product(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

#[cfg(test)]
mod tests;
