use std::path::PathBuf;

use super::DirRow;

#[derive(Debug, Clone)]
pub enum Message {
    Event(Event),
    Internal(Internal),
}

/// Events the app must act on.
#[derive(Debug, Clone)]
pub enum Event {
    /// Ask the app to start the indexing pipeline for this directory.
    CacheRequest(PathBuf),
    /// Ask the app to clear this directory's cached entries.
    ClearRequest(PathBuf),
}

/// Page-internal state changes.
#[derive(Debug, Clone)]
pub enum Internal {
    FilterInput(String),
    DirInput(String),
    RefreshPressed,
    CachePressed,
    /// Result of the async table load.
    RowsLoaded(Vec<DirRow>),
}
