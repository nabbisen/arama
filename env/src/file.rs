use std::{io::Result, path::PathBuf};

use crate::cache_dir;

pub const IMAGE_EXTENSION_ALLOWLIST: &[&str; 6] = &["png", "jpg", "jpeg", "webp", "gif", "bmp"];
pub const VIDEO_EXTENSION_ALLOWLIST: &[&str; 1] = &["mp4"];

/// Current cache database file (v2: `localcache`-backed format).
pub const CACHE_STORAGE_FILE: &str = "cache-v2.sqlite";
/// Legacy cache database file (v1: `file-feature-cache` format).
/// Kept only so the one-time migration can locate the old database.
pub const CACHE_STORAGE_FILE_V1: &str = "cache.sqlite";
pub const CACHE_THUMBNAIL_DIR: &str = "thumbnail";

pub fn cache_storage_path() -> Result<PathBuf> {
    let cache_dir = cache_dir()?;
    let path = cache_dir.join(CACHE_STORAGE_FILE);
    Ok(path.to_path_buf())
}

/// Path of the legacy (v1) cache database. Used only by the one-time
/// migration at application startup.
pub fn cache_storage_path_v1() -> Result<PathBuf> {
    let cache_dir = cache_dir()?;
    let path = cache_dir.join(CACHE_STORAGE_FILE_V1);
    Ok(path.to_path_buf())
}

pub fn cache_thumbnail_dir_path() -> Result<PathBuf> {
    let cache_dir = cache_dir()?;
    let path = cache_dir.join(CACHE_THUMBNAIL_DIR);
    Ok(path.to_path_buf())
}
