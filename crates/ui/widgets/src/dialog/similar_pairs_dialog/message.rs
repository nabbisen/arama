use super::types::SimilarPair;

#[derive(Debug, Clone)]
pub enum Message {
    EmbeddingsReady(Vec<SimilarPair>),
    MediaItemEnter(String),
    MediaItemDoubleClicked(String),
    MediaExit,
}
