use std::path::PathBuf;

use arama_ai::pipeline::score::similarity::image::find_similar_pairs_efficient;
use arama_cache::CacheConcumer;
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
            async {
                // todo: error handling
                let path_embeddings =
                    CacheConcumer::get_embeddings(dir_node).expect("failed to get embeddings");
                // todo ui sliders for these param(s): threshold (also k_neighbors ?)
                find_similar_pairs_efficient(&path_embeddings, 0.86, 50).await
            },
            message::Message::EmbeddingsReady,
        )
    }
}
