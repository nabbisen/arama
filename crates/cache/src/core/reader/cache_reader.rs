use std::sync::Arc;

use crate::core::store::cache_store::CacheStore;

/// 参照専用ハンドル。[`CacheWriter::as_reader`] で生成する。
///
/// [`CacheWriter`] への昇格はできない。
/// lookup 中に変更が検出された場合の内部 DELETE は実装詳細として許容する。
///
/// `Clone` コストは `Arc` のカウントアップのみ。rayon の各タスクに自由に配布できる。
///
/// [`CacheWriter::as_reader`]: crate::CacheWriter::as_reader
#[derive(Clone)]
pub struct CacheReader {
    pub(crate) store: Arc<CacheStore>,
}

impl CacheReader {
    pub(crate) fn new(store: Arc<CacheStore>) -> Self {
        Self { store }
    }
}
