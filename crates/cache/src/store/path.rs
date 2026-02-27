use std::path::PathBuf;

#[cfg(test)]
mod tests;

/// DB ファイルパスを解決する。
///
/// 優先順位:
/// 1. 環境変数 `ARAMA_CACHE_DB`
/// 2. `$XDG_CACHE_HOME/arama_cache/cache.db` (未設定時は `$HOME/.cache/arama_cache/cache.db`)
/// 3. `./arama_cache.db`
pub fn resolve_db_path() -> PathBuf {
    // 1. 環境変数
    if let Ok(p) = std::env::var("ARAMA_CACHE_DB") {
        return PathBuf::from(p);
    }

    // 2. XDG キャッシュディレクトリ
    let xdg_base = std::env::var("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            std::env::var("HOME")
                .map(|h| PathBuf::from(h).join(".cache"))
                .unwrap_or_else(|_| PathBuf::from(".cache"))
        });
    let xdg_path = xdg_base.join("arama_cache").join("cache.db");

    // 親ディレクトリが作れる場合はそちらを使う
    let parent = xdg_path.parent().unwrap();
    if std::fs::create_dir_all(parent).is_ok() {
        return xdg_path;
    }

    // 3. フォールバック
    PathBuf::from("./arama_cache.db")
}
