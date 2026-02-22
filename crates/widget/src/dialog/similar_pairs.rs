use std::path::PathBuf;

use arama_embedding::pipeline::infer::clip::util::find_similar_pairs_efficient;
use arama_indexer::ImageCacheManager;
use iced::Task;
use swdir::DirNode;

pub mod message;
pub mod output;
mod update;
mod view;

#[derive(Clone, Debug)]
pub struct SimilarPairs {
    dir_node: DirNode,
    pairs: Option<Vec<(PathBuf, PathBuf, f32)>>,
}

impl SimilarPairs {
    pub fn new<T: Into<DirNode>>(dir_node: T, pairs: Option<Vec<(PathBuf, PathBuf, f32)>>) -> Self {
        Self {
            dir_node: dir_node.into(),
            pairs,
        }
    }

    pub fn default_task(&self) -> Task<message::Message> {
        let dir_node = self.dir_node.clone();
        Task::perform(
            async {
                // todo: error handling
                let path_embeddings =
                    ImageCacheManager::get_embeddings(dir_node).expect("failed to get embeddings");
                find_similar_pairs_efficient(&path_embeddings, 0.75, 20).await
            },
            message::Message::EmbeddingsReady,
        )
    }
}
