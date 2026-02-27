use std::path::Path;

use crate::error::{Result, cache_error::CacheError};
use crate::identity::api::matches_stored;
use crate::store::cache_store::CacheStore;

pub fn file_matches(
    inner: &CacheStore,
    stored_hash: &str,
    stored_mtime: Option<i64>,
    path: &Path,
) -> Result<bool> {
    matches_stored(stored_hash, stored_mtime, path, &inner.config.hash_strategy)
        .map_err(|e| CacheError::io(path, e))
}
