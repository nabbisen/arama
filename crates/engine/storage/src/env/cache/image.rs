use std::{io::Result, path::PathBuf};

use crate::env::path::image::cache_thumbnail_file_path;

#[derive(Debug)]
pub struct Cache {
    #[allow(dead_code)]
    pub id: u32,
    #[allow(dead_code)]
    pub path: String,
    pub last_modified: u32,
    #[allow(dead_code)]
    pub cache_kind: u32,
    #[allow(dead_code)]
    pub embedding: Option<Vec<u8>>,
}

impl Cache {
    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn cache_file_path(&self) -> Result<PathBuf> {
        cache_thumbnail_file_path(self.id)
    }
}
