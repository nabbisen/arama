use std::path::PathBuf;

#[cfg(test)]
mod tests;

/// DB ファイルの場所を表す。`CacheConfig::db_location` に設定する。
///
/// # バリアント一覧
///
/// | バリアント | 生成パス例 |
/// |---|---|
/// | `Custom(path)` | 指定したパスそのまま |
/// | `AppCache(file_name)` | `~/.cache/<実行バイナリ名>/<file_name>` |
/// | `WorkDir(file_name)` (デフォルト) | `./<file_name>` |
#[derive(Debug, Clone)]
pub enum DbLocation {
    /// パスを完全に指定する。
    Custom(PathBuf),

    /// XDG キャッシュディレクトリを使う。
    ///
    /// アプリ名は `std::env::current_exe()` で実行時に自動取得する。
    /// `file_name` が `None` の場合は `cache.db`。
    ///
    /// `$XDG_CACHE_HOME/<実行バイナリ名>/<file_name>`
    /// (`XDG_CACHE_HOME` 未設定時は `$HOME/.cache/<実行バイナリ名>/<file_name>`)
    AppCache(Option<String>),

    /// 実行ディレクトリに作成する (デフォルト)。
    ///
    /// `file_name` が `None` の場合は `arama_cache.db`。
    WorkDir(Option<String>),
}

impl Default for DbLocation {
    fn default() -> Self {
        Self::WorkDir(None) // → ./arama_cache.db
    }
}

impl DbLocation {
    pub(crate) fn resolve(&self) -> PathBuf {
        match self {
            Self::Custom(p) => p.clone(),

            Self::AppCache(file_name) => {
                let base = std::env::var("XDG_CACHE_HOME")
                    .map(PathBuf::from)
                    .unwrap_or_else(|_| {
                        std::env::var("HOME")
                            .map(|h| PathBuf::from(h).join(".cache"))
                            .unwrap_or_else(|_| PathBuf::from(".cache"))
                    });
                let app = std::env::current_exe()
                    .ok()
                    .and_then(|p| p.file_stem().map(|s| s.to_string_lossy().into_owned()))
                    .unwrap_or_else(|| "app".to_string());
                let name = file_name.as_deref().unwrap_or("cache.db");
                base.join(app).join(name)
            }

            Self::WorkDir(file_name) => {
                let name = file_name.as_deref().unwrap_or("arama_cache.db");
                PathBuf::from(format!("./{name}"))
            }
        }
    }
}
