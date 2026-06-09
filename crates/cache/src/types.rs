//! Public types of the `arama-cache` facade.

use std::path::{Path, PathBuf};

use crate::Result;

/// Result of a cache lookup.
#[derive(Debug)]
pub enum LookupResult<T> {
    /// Cache hit: the file is unchanged and a payload was stored.
    Hit(T),
    /// The file changed since it was cached. The stored payload is no
    /// longer valid and will be replaced on the next upsert.
    Invalidated,
    /// The file is not registered in the cache.
    Miss,
}

/// Read-side capability shared by the image and video readers.
pub trait CacheRead {
    /// `true` when the cache entry for `path` exists and is fresh.
    fn check(&self, path: &Path) -> Result<bool>;
    /// Batch variant of [`check`](CacheRead::check), parallelized.
    fn check_all(&self, paths: &[&Path]) -> Vec<(PathBuf, Result<bool>)>;
    /// Canonicalized paths of every registered entry.
    fn list_paths(&self) -> Result<Vec<String>>;
}

/// Per-directory aggregate of cached entries, for cache-management UIs
/// (RFC 004).
#[derive(Debug, Clone)]
pub struct DirCacheSummary {
    /// Canonical path of the directory containing the cached files.
    pub dir_path: String,
    /// Number of cached files directly in this directory.
    pub file_count: usize,
    /// Sum of the cached files' recorded sizes, in bytes.
    pub total_size: u64,
    /// Newest `updated_at` among the entries (unix seconds).
    pub latest_cached_at: i64,
}

// ---------------------------------------------------------------------------
// Images
// ---------------------------------------------------------------------------

/// Write request for the image cache.
#[derive(Debug)]
pub struct UpsertImageRequest {
    pub path: PathBuf,
    /// CLIP feature vector (one per image). `None` preserves the stored
    /// value.
    pub clip_vector: Option<Vec<f32>>,
}

/// An image cache entry.
#[derive(Clone, Debug)]
pub struct ImageCacheEntry {
    /// Canonicalized path as stored in the database.
    pub path: String,
    /// Thumbnail file path. `None` when not generated.
    pub thumbnail_path: Option<String>,
    /// Feature vectors. `None` when not registered.
    pub features: Option<ImageFeatures>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ImageFeatures {
    pub clip_vector: Vec<f32>,
}

// ---------------------------------------------------------------------------
// Videos
// ---------------------------------------------------------------------------

/// Write request for the video cache.
#[derive(Debug)]
pub struct UpsertVideoRequest {
    pub path: PathBuf,
    /// Frame-averaged CLIP feature vector. `None` preserves the stored
    /// value.
    pub clip_vector: Option<Vec<f32>>,
    /// Scene-averaged wav2vec2 feature vector. `None` preserves the
    /// stored value.
    pub wav2vec2_vector: Option<Vec<f32>>,
}

/// A video cache entry.
#[derive(Debug)]
pub struct VideoCacheEntry {
    pub path: String,
    pub thumbnail_path: Option<String>,
    pub features: Option<VideoFeatures>,
}

#[derive(Debug, PartialEq)]
pub struct VideoFeatures {
    /// Frame-averaged CLIP feature vector.
    pub clip_vector: Option<Vec<f32>>,
    /// Scene-averaged wav2vec2 feature vector.
    pub wav2vec2_vector: Option<Vec<f32>>,
}
