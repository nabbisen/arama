use super::types::EmbeddingsReadyResult;

#[derive(Debug, Clone)]
pub enum Message {
    EmbeddingsReady(EmbeddingsReadyResult),
    MediaItemEnter(String),
    MediaItemDoubleClicked(String),
    MediaExit,
}
