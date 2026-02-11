use super::gallery_settings::{
    similarity_slider::SimilaritySlider, swdir_depth_limit::SwdirDepthLimit,
};

pub mod message;
pub mod similarity_slider;
pub mod swdir_depth_limit;
mod update;
mod view;

#[derive(Default)]
pub struct GallerySettings {
    swdir_depth_limit: SwdirDepthLimit,
    similarity_slider: SimilaritySlider,
}

impl GallerySettings {
    pub fn swdir_depth_limit(&self) -> Option<usize> {
        self.swdir_depth_limit.value
    }

    pub fn similarity_quality(&self) -> f32 {
        self.similarity_slider.similarity_quality.value()
    }
}
