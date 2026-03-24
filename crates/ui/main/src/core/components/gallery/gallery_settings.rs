use arama_env::target_media_type::TargetMediaType;

pub mod message;
mod update;
mod view;

pub struct GallerySettings {
    target_media_type: TargetMediaType,
    sub_dir_depth_limit: u8,
    embedding_cached: bool,
}

impl GallerySettings {
    pub fn new(target_media_type: &TargetMediaType, sub_dir_depth_limit: u8) -> Self {
        Self {
            target_media_type: target_media_type.to_owned(),
            sub_dir_depth_limit,
            embedding_cached: false,
        }
    }

    pub fn set_embedding_cached(&mut self, embedding_cached: bool) {
        self.embedding_cached = embedding_cached;
    }
}
