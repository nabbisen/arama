use std::path::PathBuf;

use super::{CacheLoad, CacheLoadError};

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
    /// Ask the app to prune cache storage toward this one-off byte target.
    PruneRequest(u64),
    /// Ask the app to abort the active caching run.
    StopRequest,
}

/// Page-internal state changes.
#[derive(Debug, Clone)]
pub enum Internal {
    FilterInput(String),
    DirInput(String),
    PruneTargetInput(String),
    RefreshPressed,
    CachePressed,
    PrunePressed,
    /// Result of the async table load.
    RowsLoaded(Result<CacheLoad, CacheLoadError>),
}
