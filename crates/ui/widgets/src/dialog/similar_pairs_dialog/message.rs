use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum Message {
    EmbeddingsReady(Vec<(PathBuf, PathBuf, f32)>),
    MediaItemEnter(PathBuf),
    MediaExit,
}
