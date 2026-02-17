use std::path::{Path, PathBuf};

use crate::core::store::file::file_embedding::FileEmbedding;
use naga::FastHashMap;

#[derive(Clone, Debug, Default)]
pub struct FileEmbeddingMap {
    files: FastHashMap<PathBuf, Vec<f32>>,
}

impl FileEmbeddingMap {
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    pub fn get_embedding(&self, path: &Path) -> Option<&Vec<f32>> {
        self.files.get(path)
    }

    pub fn set_embedding(&mut self, file_embedding: &FileEmbedding) {
        self.files.insert(
            file_embedding.path.to_owned(),
            file_embedding.embedding.to_owned(),
        );
    }

    pub fn similar_pairs(&self, threshold: f32) -> anyhow::Result<Vec<(PathBuf, PathBuf, f32)>> {
        crate::core::pipeline::clip::inference::clip_calculator::find_similar_pairs(
            &self.files,
            threshold,
        )
    }
}
