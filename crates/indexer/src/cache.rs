use std::{io::Result, path::PathBuf};

use arama_env::local_dir;

pub mod image;

const CACHE_DIR: &str = "cache";

pub fn caches_dir() -> Result<PathBuf> {
    let path = local_dir()?.join(CACHE_DIR);
    Ok(path.to_path_buf())
}
