use arama_repr::error::ReprError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CacheError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("arama-repr error: {0}")]
    AramaRepr(#[from] ReprError),

    #[error("connection pool error: {0}")]
    Pool(String),

    #[error("I/O error for '{path}': {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("BLOB length mismatch: expected multiple of 4 bytes for f32 vector, got {0}")]
    InvalidVectorBlob(usize),
}

impl CacheError {
    pub(crate) fn from_pool(e: r2d2::Error) -> Self {
        CacheError::Pool(e.to_string())
    }

    pub(crate) fn io(path: &std::path::Path, source: std::io::Error) -> Self {
        CacheError::Io {
            path: path.to_string_lossy().into_owned(),
            source,
        }
    }
}

pub type Result<T> = std::result::Result<T, CacheError>;
