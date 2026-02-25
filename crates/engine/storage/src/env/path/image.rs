use std::{io::Result, path::PathBuf};

use arama_env::cache_dir;

pub(super) const IMAGE_CACHE_DIR: &str = "image";
pub(super) const CACHE_THUMBNAIL_DIR: &str = "thumbnail";
pub const DATABASE_FILE: &str = "cache.image.sqlite3";

pub fn image_cache_dir() -> Result<PathBuf> {
    let path = cache_dir()?.join(IMAGE_CACHE_DIR);
    Ok(path)
}

pub fn cache_thumbnail_dir() -> Result<PathBuf> {
    let path = image_cache_dir()?.join(CACHE_THUMBNAIL_DIR);
    Ok(path)
}

pub fn cache_thumbnail_file_path(id: u32) -> Result<PathBuf> {
    Ok(cache_thumbnail_dir()?.join(&format!("{}.png", id)))
}
