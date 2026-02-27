use crate::identity::hash::hash_strategy::HashStrategy;

// ---------------------------------------------------------------------------
// CacheConfig
// ---------------------------------------------------------------------------

/// [`CacheWriter::open`] に渡す設定。
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// 読み取りプールのコネクション数。`None` で論理 CPU 数を使う。
    pub read_conns: Option<u32>,
    /// ファイル同一性確認の戦略。
    pub hash_strategy: HashStrategy,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            read_conns: None,
            hash_strategy: HashStrategy::default(),
        }
    }
}
