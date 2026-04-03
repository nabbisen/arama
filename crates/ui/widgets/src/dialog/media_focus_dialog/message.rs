use std::path::PathBuf;

use arama_env::cache_lookup_strategy::CacheLookupStrategy;

use super::types::SimilarMediaItem;

#[derive(Debug, Clone)]
pub enum Message {
    SimilarMediaReady(Vec<SimilarMediaItem>),
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
