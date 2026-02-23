use std::{io::Result, path::PathBuf};

use crate::env::path::caches_dir;

pub(super) const CACHE_DIR: &str = "image";
pub(super) const CACHE_THUMBNAIL_DIR: &str = "thumbnail";
pub const DATABASE_FILE: &str = "cache.image.sqlite3";

pub fn cache_dir() -> Result<PathBuf> {
    let path = caches_dir()?.join(CACHE_DIR);
    Ok(path)
}

pub fn cache_thumbnail_dir() -> Result<PathBuf> {
    let path = cache_dir()?.join(CACHE_THUMBNAIL_DIR);
    Ok(path)
}

pub fn cache_thumbnail_file_path(id: u32) -> Result<PathBuf> {
    Ok(cache_thumbnail_dir()?.join(&format!("{}.png", id)))
}
