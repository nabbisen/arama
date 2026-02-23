use crate::core::components::gallery::gallery_settings::thumbnail_size_slider::ThumbnailSizeSlider;

pub mod media_type;
pub mod message;
pub mod output;
pub mod thumbnail_size_slider;
mod update;
mod view;

use media_type::MediaType;

#[derive(Default)]
pub struct GallerySettings {
    media_type: MediaType,
    thumbnail_size: ThumbnailSizeSlider,
}

impl GallerySettings {
    pub fn thumbnail_size(&self) -> u16 {
        self.thumbnail_size.value
    }
}
