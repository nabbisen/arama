use std::{io::Result, path::PathBuf};

use crate::cache_dir;

pub const IMAGE_EXTENSION_ALLOWLIST: &[&str; 6] = &["png", "jpg", "jpeg", "webp", "gif", "bmp"];
pub const VIDEO_EXTENSION_ALLOWLIST: &[&str; 1] = &["mp4"];

pub const CACHE_STORAGE_FILE: &str = "cache.sqlite";

pub fn cache_storage_path() -> Result<PathBuf> {
    let cache_dir = cache_dir()?;
    let path = cache_dir
        .parent()
        .expect("failed to get cache parent directory")
        .join(CACHE_STORAGE_FILE);
    Ok(path.to_path_buf())
}
