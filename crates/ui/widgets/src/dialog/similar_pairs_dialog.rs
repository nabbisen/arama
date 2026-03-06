use std::path::PathBuf;

use arama_ai::pipeline::score::similarity::image::find_similar_pairs_efficient;
use arama_cache::{CacheConfig, DbLocation, ImageCacheConfig, ImageCacheReader, LookupResult};
use arama_env::{cache_storage_path, cache_thumbnail_dir_path};
use iced::Task;
use swdir::DirNode;

pub mod message;
pub mod output;
mod update;
mod view;

#[derive(Clone, Debug)]
pub struct SimilarPairsDialog {
    dir_node: DirNode,
    pairs: Option<Vec<(PathBuf, PathBuf, f32)>>,
}

impl SimilarPairsDialog {
    pub fn new<T: Into<DirNode>>(dir_node: T, pairs: Option<Vec<(PathBuf, PathBuf, f32)>>) -> Self {
        Self {
            dir_node: dir_node.into(),
            pairs,
        }
    }

    pub fn default_task(&self) -> Task<message::Message> {
        let dir_node = self.dir_node.clone();
        Task::perform(
            async move {
                let db_location = DbLocation::Custom(
                    cache_storage_path().expect("failed to get cache stogate path"),
                );
                let cache_reader = ImageCacheReader::as_session(ImageCacheConfig {
                    cache: CacheConfig {
                        db_location,
                        read_conns: 4,
                        thumbnail_dir: Some(
                            cache_thumbnail_dir_path()
                                .expect("failed to get cache thumbnail dir path"),
                        ),
                    },
                })
                .expect("failed to get cache writer");

                let paths = dir_node.flatten_paths();

                let mut path_embeddings: Vec<(PathBuf, Vec<f32>)> = vec![];
                for path in paths {
                    let feature = match cache_reader.lookup(&path).expect("failed to lookup") {
                        LookupResult::Hit(x) => Some((
                            PathBuf::from(x.thumbnail_path.expect("failed to get thumbnail path")),
                            x.features.expect("failed to get feature").clip_vector,
                        )),
                        _ => {
                            // todo: error handling
                            None
                        }
                    };

                    if let Some(feature) = feature {
                        path_embeddings.push(feature);
                    }
                }

                // todo ui sliders for these param(s): threshold (also k_neighbors ?)
                find_similar_pairs_efficient(&path_embeddings, 0.86, 50).await
            },
            message::Message::EmbeddingsReady,
        )
    }
}
