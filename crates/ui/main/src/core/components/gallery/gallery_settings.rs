use arama_env::target_media_type::TargetMediaType;

use crate::core::components::gallery::gallery_settings::thumbnail_size_slider::ThumbnailSizeSlider;

pub mod message;
pub mod thumbnail_size_slider;
mod update;
mod view;

pub struct GallerySettings {
    target_media_type: TargetMediaType,
    sub_dir_depth_limit: u8,
    thumbnail_size: ThumbnailSizeSlider,
    embedding_cached: bool,
}

impl GallerySettings {
    pub fn new(target_media_type: &TargetMediaType, sub_dir_depth_limit: u8) -> Self {
        Self {
            target_media_type: target_media_type.to_owned(),
            sub_dir_depth_limit,
            thumbnail_size: ThumbnailSizeSlider::default(),
            embedding_cached: false,
        }
    }

    pub fn thumbnail_size(&self) -> u16 {
        self.thumbnail_size.value
    }

    pub fn set_embedding_cached(&mut self, embedding_cached: bool) {
        self.embedding_cached = embedding_cached;
    }
}
