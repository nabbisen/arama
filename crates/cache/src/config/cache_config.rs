use super::db_location::DbLocation;
use crate::core::identity::hash::hash_strategy::HashStrategy;

/// [`CacheWriter::open`] / [`CacheWriter::open_with_config`] に渡す設定。
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// DB ファイルの場所。デフォルトは `DbLocation::WorkDir` (`./arama_cache.db`)。
    pub db_location: DbLocation,
    /// 読み取りプールのコネクション数。`None` で論理 CPU 数を使う。
    pub read_conns: Option<u32>,
    /// ファイル同一性確認の戦略。
    pub hash_strategy: HashStrategy,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            db_location: DbLocation::default(),
            read_conns: None,
            hash_strategy: HashStrategy::default(),
        }
    }
}
