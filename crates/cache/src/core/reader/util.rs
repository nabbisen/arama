use crate::core::identity::api::matches_stored;
use crate::core::store::cache_store::CacheStore;

/// DB の `(stored_hash, stored_mtime)` と現在のファイルを比較する。
/// `CacheReader` の lookup と `CacheWriter` の verify_or_invalidate の両方から使う。
pub fn file_matches(
    inner: &CacheStore,
    stored_hash: &str,
    stored_mtime: Option<i64>,
    path: &std::path::Path,
) -> crate::error::Result<bool> {
    matches_stored(stored_hash, stored_mtime, path, &inner.config.hash_strategy)
        .map_err(|e| crate::error::CacheError::io(path, e))
}
