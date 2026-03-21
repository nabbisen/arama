use std::path::PathBuf;

use arama_ai::pipeline::score::similarity::image::find_similar_pairs;
use arama_cache::{
    CacheConfig, DbLocation, ImageCacheConfig, ImageCacheReader, LookupResult, VideoCacheConfig,
    VideoCacheReader,
};
use arama_env::{
    IMAGE_EXTENSION_ALLOWLIST, MIN_IMAGE_SIMILARITY, MIN_VIDEO_SIMILARITY,
    VIDEO_EXTENSION_ALLOWLIST, cache_storage_path, cache_thumbnail_dir_path,
};
use arama_sidecar::media::video::video_engine::VideoEngine;
use iced::Task;
use swdir::DirNode;

pub mod message;
mod types;
mod update;
mod view;

use types::SimilarPair;

use crate::dialog::similar_pairs_dialog::types::SimilarPairItem;

#[derive(Clone, Debug)]
pub struct SimilarPairsDialog {
    dir_node: DirNode,
    pairs: Option<Vec<SimilarPair>>,
    hovered_media_item_path_str: Option<String>,
}

impl SimilarPairsDialog {
    pub fn new<T: Into<DirNode>>(dir_node: T, pairs: Option<Vec<SimilarPair>>) -> Self {
        Self {
            dir_node: dir_node.into(),
            pairs,
            hovered_media_item_path_str: None,
        }
    }

    pub fn default_task(&self) -> Task<message::Message> {
        let dir_node = self.dir_node.clone();
        Task::perform(
            prepare_embeddings(dir_node),
            message::Message::EmbeddingsReady,
        )
    }
}

async fn prepare_embeddings(dir_node: DirNode) -> Vec<SimilarPair> {
    let paths = dir_node.flatten_paths();

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

    let mut image_path_embeddings: Vec<(String, Option<String>, Vec<f32>)> = vec![];
    let image_paths: Vec<&PathBuf> = paths
        .iter()
        .filter(|x| {
            x.extension().is_some_and(|x| {
                IMAGE_EXTENSION_ALLOWLIST.contains(&x.to_string_lossy().to_string().as_str())
            })
        })
        .collect();
    if 0 < image_paths.len() {
        let image_cache_reader = ImageCacheReader::as_session(ImageCacheConfig {
            cache_config: cache_config.clone(),
        })
        .expect("failed to get image cache writer");

        for path in &image_paths {
            let feature = match image_cache_reader.lookup(&path).expect("failed to lookup") {
                LookupResult::Hit(x) => Some((
                    x.path,
                    x.thumbnail_path,
                    x.features.expect("failed to get feature").clip_vector,
                )),
                _ => {
                    // todo: error handling
                    None
                }
            };

            if let Some(feature) = feature {
                image_path_embeddings.push(feature);
            }
        }
    }

    let mut video_path_embeddings: Vec<(String, Option<String>, Vec<f32>)> = vec![];
    let video_paths: Vec<&PathBuf> = paths
        .iter()
        .filter(|x| {
            x.extension().is_some_and(|x| {
                VIDEO_EXTENSION_ALLOWLIST.contains(&x.to_string_lossy().to_string().as_str())
            })
        })
        .collect();
    if 0 < video_paths.len() {
        let video_cache_reader = VideoCacheReader::as_session(VideoCacheConfig {
            cache_config,
            ffmpeg_path: Some(VideoEngine::ffmpeg_path().expect("failed to get ffmpeg path")),
        })
        .expect("failed to get video cache writer");

        for path in &video_paths {
            let feature = match video_cache_reader.lookup(&path).expect("failed to lookup") {
                LookupResult::Hit(x) => Some((
                    x.path,
                    x.thumbnail_path,
                    x.features
                        .expect("failed to get feature")
                        .clip_vector
                        .expect("failed to get video clip embedding list"),
                )),
                _ => {
                    // todo: error handling
                    None
                }
            };

            if let Some(feature) = feature {
                video_path_embeddings.push(feature);
            }
        }
    }

    // todo ui sliders for these param(s): threshold (also k_neighbors ?)
    let mut similar_pairs =
        find_similar_pairs(&image_path_embeddings, MIN_IMAGE_SIMILARITY, 50).await;
    let video_pairs = find_similar_pairs(&video_path_embeddings, MIN_VIDEO_SIMILARITY, 50).await;
    similar_pairs.extend(video_pairs);
    similar_pairs
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
        .collect()
}
