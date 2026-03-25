pub mod message;
mod update;
mod view;

pub struct GallerySettings {
    embedding_cached: bool,
}

impl GallerySettings {
    pub fn new() -> Self {
        Self {
            embedding_cached: false,
        }
    }

    pub fn set_embedding_cached(&mut self, embedding_cached: bool) {
        self.embedding_cached = embedding_cached;
    }
}
