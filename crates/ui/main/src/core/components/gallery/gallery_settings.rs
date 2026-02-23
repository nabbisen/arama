use crate::core::components::gallery::gallery_settings::thumbnail_size_slider::ThumbnailSizeSlider;

pub mod message;
pub mod output;
pub mod target_media_type;
pub mod thumbnail_size_slider;
mod update;
mod view;

use target_media_type::TargetMediaType;

#[derive(Default)]
pub struct GallerySettings {
    target_media_type: TargetMediaType,
    thumbnail_size: ThumbnailSizeSlider,
}

impl GallerySettings {
    pub fn thumbnail_size(&self) -> u16 {
        self.thumbnail_size.value
    }
}
