use thiserror::Error;

#[derive(Debug, Error)]
pub enum CacheError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("connection pool error: {0}")]
    Pool(String),

    #[error("I/O error for '{path}': {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("BLOB 長が不正: f32 ベクトルは 4 の倍数バイト必要, got {0} bytes")]
    InvalidVectorBlob(usize),

    #[error("thumbnail_dir が未設定です; CacheConfig::thumbnail_dir を設定してください")]
    ThumbnailDirNotConfigured,

    #[error("サムネイル生成エラー: {0}")]
    ThumbnailGenerationFailed(String),
}

impl CacheError {
    pub(crate) fn pool(e: r2d2::Error) -> Self {
        CacheError::Pool(e.to_string())
    }

    pub(crate) fn io(path: &std::path::Path, source: std::io::Error) -> Self {
        CacheError::Io {
            path: path.to_string_lossy().into_owned(),
            source,
        }
    }

    pub(crate) fn io_str(path: &str, source: std::io::Error) -> Self {
        CacheError::Io {
            path: path.to_owned(),
            source,
        }
    }
}

pub type Result<T> = std::result::Result<T, CacheError>;
