use std::{io::Result, path::PathBuf};

use arama_env::cache_dir;

pub(super) const AUDIO_CACHE_DIR: &str = "audio";

pub fn audio_cache_dir() -> Result<PathBuf> {
    let path = cache_dir()?.join(AUDIO_CACHE_DIR);
    Ok(path)
}
