use std::path::PathBuf;

pub mod message;
mod view;

#[derive(Clone, Debug)]
pub struct ImageCell {
    path: PathBuf,
    thumbnail_path: PathBuf,
    thumbnail_size: u32,
}

impl ImageCell {
    pub fn new<T: Into<PathBuf>>(path: T, thumbnail_path: T, thumbnail_size: u32) -> Self {
        Self {
            path: path.into(),
            thumbnail_path: thumbnail_path.into(),
            thumbnail_size,
        }
    }
}
