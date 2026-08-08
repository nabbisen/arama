use crate::dialog::similarity_read_outcome::SimilarityReadOutcome;

use super::types::SimilarPair;

#[derive(Debug, Clone)]
pub enum Message {
    EmbeddingsReady(SimilarityReadOutcome<SimilarPair>),
    MediaItemEnter(String),
    MediaItemDoubleClicked(String),
    MediaExit,
}
