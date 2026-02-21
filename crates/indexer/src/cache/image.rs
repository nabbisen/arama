use std::{io::Result, path::PathBuf};

mod byte;
mod database;
pub mod image_cache_manager;
mod path;

use path::cache_thumbnail_file_path;

#[derive(Debug)]
pub struct Cache {
    #[allow(dead_code)]
    id: u32,
    #[allow(dead_code)]
    path: String,
    last_modified: u32,
    #[allow(dead_code)]
    cache_kind: u32,
    #[allow(dead_code)]
    embedding: Option<Vec<u8>>,
}

impl Cache {
    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn cache_file_path(&self) -> Result<PathBuf> {
        cache_thumbnail_file_path(self.id)
    }
}

enum CacheKind {
    Image,
}
