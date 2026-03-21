use std::path::PathBuf;

use super::types::SimilarMediaItem;

#[derive(Debug, Clone)]
pub enum Message {
    SimilarMediaReady(Vec<SimilarMediaItem>),
    SimilarMediaItemDoubleClicked(PathBuf),
    HistoryPrevious,
    HistoryNext,
    MediaItemEnter(String),
    MediaItemExit,
    ViewSizeToggle(bool),
    CloseClick,
}
