pub mod message;
mod update;
mod view;

pub struct GallerySettings {
    sub_dir_depth_limit: u8,
    embedding_cached: bool,
}

impl GallerySettings {
    pub fn new(sub_dir_depth_limit: u8) -> Self {
        Self {
            sub_dir_depth_limit,
            embedding_cached: false,
        }
    }

    pub fn set_embedding_cached(&mut self, embedding_cached: bool) {
        self.embedding_cached = embedding_cached;
    }
}
