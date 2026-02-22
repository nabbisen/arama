use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum Output {
    EmbeddingsReady(Vec<(PathBuf, PathBuf, f32)>),
}
