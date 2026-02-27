use std::path::Path;

use crate::core::identity::api::matches_stored;
use crate::core::store::cache_store::CacheStore;
use crate::error::{CacheError, Result};

pub fn file_matches(
    inner: &CacheStore,
    stored_hash: &str,
    stored_mtime: Option<i64>,
    path: &Path,
) -> Result<bool> {
    matches_stored(stored_hash, stored_mtime, path, &inner.config.hash_strategy)
        .map_err(|e| CacheError::io(path, e))
}
