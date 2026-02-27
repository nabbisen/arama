// ---------------------------------------------------------------------------
// 設定型
// ---------------------------------------------------------------------------

use crate::identity::hash_strategy::HashStrategy;

/// `CacheStore::open` に渡す設定。
#[derive(Debug, Clone)]
pub struct CacheStoreConfig {
    /// 読み取りプールのコネクション数。`None` の場合は論理 CPU 数を使う。
    pub read_conns: Option<u32>,
    /// ファイル同一性確認の戦略。
    pub hash_strategy: HashStrategy,
}

impl Default for CacheStoreConfig {
    fn default() -> Self {
        Self {
            read_conns: None,
            hash_strategy: HashStrategy::default(),
        }
    }
}
