use std::{fs::create_dir_all, path::PathBuf};

use crate::CacheConfig;

#[cfg(test)]
mod tests;

/// DB ファイルパスを解決する。
///
/// 優先順位:
/// 1. `config.db_path` が `Some` ならそのパス
/// 2. 環境変数 `arama_cache_DB`
/// 3. `$XDG_CACHE_HOME/arama_cache/cache.db` (未設定時は `$HOME/.cache/arama_cache/cache.db`)
/// 4. `./arama_cache.db`
pub(crate) fn resolve_db_path(config: &CacheConfig) -> PathBuf {
    // 1. CacheConfig::db_path
    if let Some(ref p) = config.db_path {
        return p.clone();
    }

    // 2. 環境変数
    if let Ok(p) = std::env::var("arama_cache_DB") {
        return PathBuf::from(p);
    }

    // 3. XDG キャッシュディレクトリ
    let xdg_base = std::env::var("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            std::env::var("HOME")
                .map(|h| PathBuf::from(h).join(".cache"))
                .unwrap_or_else(|_| PathBuf::from(".cache"))
        });
    let xdg_path = xdg_base.join("arama_cache").join("cache.db");

    let parent = xdg_path.parent().unwrap();
    if create_dir_all(parent).is_ok() {
        return xdg_path;
    }

    // 4. フォールバック
    PathBuf::from("./arama_cache.db")
}
