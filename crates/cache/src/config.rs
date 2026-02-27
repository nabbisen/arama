use std::path::PathBuf;

use crate::core::identity::hash::hash_strategy::HashStrategy;

// ---------------------------------------------------------------------------
// CacheConfig
// ---------------------------------------------------------------------------

/// [`CacheWriter::open`] / [`CacheWriter::open_with_config`] に渡す設定。
///
/// ## DB ファイルパスの解決順序
///
/// 1. `db_path` が `Some` ならそのパスを使う  ← アプリが明示指定する主経路
/// 2. 環境変数 `arama_cache_DB` が設定されていればそのパス
/// 3. `$XDG_CACHE_HOME/arama_cache/cache.db`
///    (未設定時は `$HOME/.cache/arama_cache/cache.db`)
/// 4. カレントディレクトリの `./arama_cache.db`
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// DB ファイルパス。`None` の場合は環境変数 → XDG → フォールバックで自動解決する。
    pub db_path: Option<PathBuf>,
    /// 読み取りプールのコネクション数。`None` で論理 CPU 数を使う。
    pub read_conns: Option<u32>,
    /// ファイル同一性確認の戦略。
    pub hash_strategy: HashStrategy,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            db_path: None,
            read_conns: None,
            hash_strategy: HashStrategy::default(),
        }
    }
}
