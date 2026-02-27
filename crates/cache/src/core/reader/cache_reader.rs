use std::sync::Arc;

use crate::core::store::cache_store::CacheStore;

/// 参照専用ハンドル。[`CacheWriter::as_reader`] で生成する。
///
/// [`CacheWriter`] への昇格はできない。
///
/// [`CacheWriter::as_reader`]: crate::CacheWriter::as_reader
#[derive(Clone)]
pub struct CacheReader {
    pub(crate) inner: Arc<CacheStore>,
}

impl CacheReader {
    pub(crate) fn new(inner: Arc<CacheStore>) -> Self {
        Self { inner }
    }
}
