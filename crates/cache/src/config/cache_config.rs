use crate::identity::hash::hash_strategy::HashStrategy;

// ---------------------------------------------------------------------------
// CacheConfig
// ---------------------------------------------------------------------------

/// [`CacheWriter::open`] に渡す設定。
///
/// DB ファイルのパスはここには含まない。
/// パスの決定は以下の優先順位でライブラリが自動解決する。
///
/// 1. 環境変数 `ARAMA_CACHE_DB` が設定されていればそのパス
/// 2. `$XDG_CACHE_HOME/arama_cache/cache.db`
///    (未設定時は `$HOME/.cache/arama_cache/cache.db`)
/// 3. カレントディレクトリの `./arama_cache.db`
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
