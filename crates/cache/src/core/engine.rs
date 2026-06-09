//! Engine-facing configuration and helpers.
//!
//! This module owns the vocabulary that used to come from the in-house
//! `file-feature-cache` engine ([`DbLocation`], [`CacheConfig`],
//! [`CacheError`], [`Result`]) and the glue that maps it onto
//! [`localcache`] (RFC 002).
//!
//! Design choices (see `rfcs/done/002-replace-cache-engine-with-localcache.md`):
//!
//! - One SQLite database file, two namespaces (`"image"` / `"video"`).
//! - Change detection: [`ChangeDetectionMode::MetadataThenFullHash`] —
//!   unchanged files are verified by `mtime` + size alone (no hashing),
//!   changed files are confirmed with a full BLAKE3 hash.
//!   `localcache` has stored `mtime` at nanosecond precision since v0.20,
//!   so there is no same-second blind window.
//! - Payload versioning: bump the constants below whenever the embedding
//!   pipeline or the thumbnail format changes in a way that invalidates
//!   stored payloads. Entries with a stale version are purged when a
//!   writer opens the cache.

use std::path::{Path, PathBuf};

use localcache::{CacheEngine, CacheOptions, CacheStatus, ChangeDetectionMode};
use serde::{Serialize, de::DeserializeOwned};

// ---------------------------------------------------------------------------
// Namespaces and payload versions
// ---------------------------------------------------------------------------

/// Namespace for image entries inside the shared cache database.
pub(crate) const NAMESPACE_IMAGE: &str = "image";
/// Namespace for video entries inside the shared cache database.
pub(crate) const NAMESPACE_VIDEO: &str = "video";

/// Version of the image payload layout / pipeline. Bump to invalidate.
pub(crate) const IMAGE_PAYLOAD_VERSION: u32 = 1;
/// Version of the video payload layout / pipeline. Bump to invalidate.
pub(crate) const VIDEO_PAYLOAD_VERSION: u32 = 1;

// ---------------------------------------------------------------------------
// Error / Result
// ---------------------------------------------------------------------------

/// Errors produced by the `arama-cache` facade.
#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    /// Error bubbled up from the `localcache` engine.
    #[error("cache engine error: {0}")]
    Engine(#[from] localcache::LocalFileCacheError),

    /// Thumbnail generation failed (image decode, resize, or ffmpeg).
    #[error("thumbnail generation failed: {0}")]
    ThumbnailGenerationFailed(String),

    /// Filesystem error with the offending path attached.
    #[error("I/O error for '{path}': {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },

    /// One-time migration from the v1 cache database failed.
    #[error("v1 cache migration failed: {0}")]
    Migration(String),
}

impl CacheError {
    pub(crate) fn io(path: &Path, source: std::io::Error) -> Self {
        CacheError::Io {
            path: path.to_string_lossy().into_owned(),
            source,
        }
    }
}

/// Convenience alias used across the crate and by consumers.
pub type Result<T> = std::result::Result<T, CacheError>;

// ---------------------------------------------------------------------------
// DbLocation
// ---------------------------------------------------------------------------

/// Where the cache database file lives.
#[derive(Debug, Clone)]
pub enum DbLocation {
    /// Fully specified path.
    Custom(PathBuf),

    /// XDG cache directory: `$XDG_CACHE_HOME/<app>/<name>`.
    ///
    /// `name` defaults to `cache.db` when `None`. The application name is
    /// derived from the executable file name.
    AppCache(Option<String>),

    /// Current working directory: `./<name>`.
    ///
    /// `name` defaults to `cache.db` when `None`.
    WorkDir(Option<String>),
}

impl Default for DbLocation {
    fn default() -> Self {
        Self::WorkDir(None)
    }
}

impl DbLocation {
    /// Resolve to a concrete filesystem path.
    pub fn resolve(&self) -> PathBuf {
        match self {
            Self::Custom(p) => p.clone(),

            Self::AppCache(name) => {
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
                base.join(app).join(name.as_deref().unwrap_or("cache.db"))
            }

            Self::WorkDir(name) => {
                PathBuf::from(format!("./{}", name.as_deref().unwrap_or("cache.db")))
            }
        }
    }
}

// ---------------------------------------------------------------------------
// CacheConfig
// ---------------------------------------------------------------------------

/// Session-level configuration shared by image and video handles.
#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub db_location: DbLocation,
    /// Number of read-only connections in the read pool. Align with the
    /// expected level of read parallelism (e.g. the rayon thread count).
    pub read_conns: u32,
    /// Directory where thumbnail files are stored. `None` disables
    /// thumbnail management entirely.
    pub thumbnail_dir: Option<PathBuf>,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            db_location: DbLocation::default(),
            read_conns: num_cpus(),
            thumbnail_dir: None,
        }
    }
}

fn num_cpus() -> u32 {
    std::thread::available_parallelism()
        .map(|n| n.get() as u32)
        .unwrap_or(4)
}

// ---------------------------------------------------------------------------
// localcache glue
// ---------------------------------------------------------------------------

/// Create the parent directory of the database file if it does not yet
/// exist.
///
/// `localcache` — and SQLite underneath it — does not create intermediate
/// directories. When they are absent, `Connection::open` fails with
/// `SQLITE_CANTOPEN (14)` ("unable to open database file"). The previous
/// `file-feature-cache` engine called `validate_dir` before opening;
/// this helper restores that guarantee.
pub(crate) fn ensure_db_dir(options: &CacheOptions) -> Result<()> {
    if let Some(parent) = options.database_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| CacheError::io(parent, e))?;
    }
    Ok(())
}

/// Build the [`CacheOptions`] shared by all handles for one namespace.
pub(crate) fn cache_options(
    config: &CacheConfig,
    namespace: &str,
    payload_version: u32,
) -> CacheOptions {
    CacheOptions {
        database_path: config.db_location.resolve(),
        change_detection_mode: ChangeDetectionMode::MetadataThenFullHash,
        namespace: namespace.to_owned(),
        payload_version,
        ..CacheOptions::default()
    }
}

/// Ensure the database file exists and its schema is initialized.
///
/// Read-only connections (the read pool) skip schema creation, so a
/// standalone reader on a never-written database would fail its first
/// query. Opening (and immediately dropping) one writable engine first
/// creates the file and runs migrations — the same pattern `localcache`'s
/// own `ReadPool` tests use.
pub(crate) fn ensure_schema<T>(options: &CacheOptions) -> Result<()>
where
    T: Serialize + DeserializeOwned,
{
    let _engine: CacheEngine<T> = CacheEngine::open(options.clone())?;
    Ok(())
}

/// Number of read-pool slots for a config (`read_conns`, at least 1).
pub(crate) fn read_pool_size(config: &CacheConfig) -> usize {
    (config.read_conns.max(1)) as usize
}

// ---------------------------------------------------------------------------
// Status mapping
// ---------------------------------------------------------------------------

/// Map a `localcache` freshness status onto the legacy three-state
/// decision used by [`crate::types::LookupResult`]:
///
/// | `CacheStatus` | meaning here |
/// |---|---|
/// | `Missing` | no entry (or the file itself is gone) → `Miss` |
/// | `Stale` | entry exists but the file changed → `Invalidated` |
/// | `Fresh` | entry is valid → `Hit` (caller loads the payload) |
pub(crate) fn is_fresh(status: &CacheStatus) -> bool {
    matches!(status, CacheStatus::Fresh)
}
