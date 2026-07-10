use std::path::Path;

use arama_ai::config::video_similarity_config::VideoSimilarityConfig;
use arama_sidecar::media::video::video_engine::VideoEngine;
use rayon::prelude::*;

use arama_cache::{
    CacheConfig, DbLocation, ImageCacheConfig, ImageCacheReader, VideoCacheConfig, VideoCacheReader,
};
use arama_env::{
    VIDEO_EXTENSION_ALLOWLIST, cache_lookup_strategy::CacheLookupStrategy, cache_storage_path,
    cache_thumbnail_dir_path,
};

use super::{MediaFocusDialog, types::SimilarMediaItem};

impl MediaFocusDialog {
    pub fn similar_media(&self) -> Vec<SimilarMediaItem> {
        let threshold = self.similarity_threshold;
        let path = &self.history[self.history_index];

        let db_location =
            DbLocation::Custom(cache_storage_path().expect("failed to get cache stogate path"));
        let read_conns = 4;
        let thumbnail_dir =
            Some(cache_thumbnail_dir_path().expect("failed to get cache thumbnail dir path"));

        let cache_config = CacheConfig {
            db_location,
            read_conns,
            thumbnail_dir,
        };

        let is_video = path.extension().is_some_and(|x| {
            VIDEO_EXTENSION_ALLOWLIST.contains(&x.to_string_lossy().to_string().as_str())
        });

        if is_video {
            similar_videos(path, cache_config, self.cache_lookup_strategy, threshold)
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
) -> Vec<SimilarMediaItem> {
    let image_cache_reader = ImageCacheReader::as_session(ImageCacheConfig { cache_config })
        .expect("failed to get image cache writer");

    let cache_lookuped = match cache_lookup_strategy {
        CacheLookupStrategy::Everywhere => image_cache_reader.all(),
        CacheLookupStrategy::CurrentDirAndSubDirs => {
            image_cache_reader.all_in_dir_and_sub_dirs(path)
        }
        CacheLookupStrategy::CurrentDirOnly => image_cache_reader.all_in_dir(path),
    };

    let cache_entries = cache_lookuped
        .expect("failed to lookup")
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

    // Split target and candidate entries.
    let canonical_path = path
        .canonicalize()
        .expect("failed to canonicalize")
        .to_string_lossy()
        .to_string();
    let (target_item, candidates): (Vec<_>, Vec<_>) = cache_entries
        .into_iter()
        .partition(|x| x.path == canonical_path);

    let target_clip_vector = if let Some(features) = &target_item[0].features {
        features.clip_vector.to_owned()
    } else {
        return vec![];
    };

    // Compute similarities in parallel.
    let mut ret = candidates
        .into_par_iter()
        .map(|x| {
            let similarity = dot_product(
                &target_clip_vector,
                &x.features.expect("failed to get features").clip_vector,
            );
            SimilarMediaItem {
                path: x.path,
                thumbnail_path: x.thumbnail_path,
                similarity,
            }
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

    ret
}

fn similar_videos(
    path: &Path,
    cache_config: CacheConfig,
    cache_lookup_strategy: CacheLookupStrategy,
    threshold: f32,
) -> Vec<SimilarMediaItem> {
    let video_cache_reader = VideoCacheReader::as_session(VideoCacheConfig {
        cache_config,
        ffmpeg_path: Some(VideoEngine::ffmpeg_path().expect("failed to get ffmpeg path")),
    })
    .expect("failed to get video cache writer");

    let cache_lookuped = match cache_lookup_strategy {
        CacheLookupStrategy::Everywhere => video_cache_reader.all(),
        CacheLookupStrategy::CurrentDirAndSubDirs => {
            video_cache_reader.all_in_dir_and_sub_dirs(path)
        }
        CacheLookupStrategy::CurrentDirOnly => video_cache_reader.all_in_dir(path),
    };

    let cache_entries = cache_lookuped
        .expect("failed to lookup")
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

    // Split target and candidate entries.
    let canonical_path = path
        .canonicalize()
        .expect("failed to canonicalize")
        .to_string_lossy()
        .to_string();
    let (target_item, candidates): (Vec<_>, Vec<_>) = cache_entries
        .into_iter()
        .partition(|x| x.path == canonical_path);

    let target_features = if target_item[0].features.is_none()
        || target_item[0]
            .features
            .as_ref()
            .is_some_and(|x| x.clip_vector.is_none() || x.wav2vec2_vector.is_none())
    {
        return vec![];
    } else {
        target_item[0].features.as_ref().unwrap()
    };

    // Compute similarities in parallel.
    let mut ret = candidates
        .into_par_iter()
        .map(|x| {
            let similarity = match &x.features {
                Some(features)
                    if features.clip_vector.is_some() && features.wav2vec2_vector.is_some() =>
                {
                    let image_similarity = dot_product(
                        target_features.clip_vector.as_ref().unwrap(),
                        features.clip_vector.as_ref().unwrap(),
                    );

                    let audio_similarity = dot_product(
                        target_features.wav2vec2_vector.as_ref().unwrap(),
                        features.wav2vec2_vector.as_ref().unwrap(),
                    );

                    let video_similarity_config = VideoSimilarityConfig::default();
                    image_similarity * video_similarity_config.image_weight
                        + audio_similarity * video_similarity_config.audio_weight
                }
                _ => 0.0,
            };

            SimilarMediaItem {
                path: x.path,
                thumbnail_path: x.thumbnail_path,
                similarity,
            }
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

    ret
}

/// Calculate the dot product of two vectors. For normalized vectors, this is
/// cosine similarity.
fn dot_product(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}
