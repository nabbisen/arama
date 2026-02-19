use crate::core::components::gallery::gallery_settings::thumbnail_size_slider::ThumbnailSizeSlider;

pub mod message;
pub mod thumbnail_size_slider;
mod update;
mod view;

#[derive(Default)]
pub struct GallerySettings {
    thumbnail_size: ThumbnailSizeSlider,
}

impl GallerySettings {
    pub fn thumbnail_size(&self) -> u16 {
        self.thumbnail_size.value
    }
}
