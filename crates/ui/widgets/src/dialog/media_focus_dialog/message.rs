use std::path::PathBuf;

use arama_env::cache_lookup_strategy::CacheLookupStrategy;

use crate::dialog::similarity_read_outcome::SimilarityReadOutcome;

use super::types::SimilarMediaItem;

#[derive(Debug, Clone)]
pub enum Message {
    SimilarMediaReady(SimilarityReadOutcome<SimilarMediaItem>),
    SimilarMediaItemDoubleClicked(PathBuf),
    HistoryPrevious,
    HistoryNext,
    OpenWithDefault,
    FileManagerShow,
    CacheLookupStrategyChanged(CacheLookupStrategy),
    MediaItemEnter(String),
    MediaItemExit,
    ViewSizeToggle,
    CloseClick,
}
