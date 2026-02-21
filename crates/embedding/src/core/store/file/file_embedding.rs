use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct FileEmbedding {
    pub path: PathBuf,
    pub embedding: Vec<f32>,
}
