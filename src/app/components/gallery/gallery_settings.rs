use super::gallery_settings::similarity_slider::SimilaritySlider;

pub mod message;
pub mod similarity_slider;
mod update;
mod view;

#[derive(Default)]
pub struct GallerySettings {
    similarity_slider: SimilaritySlider,
}

impl GallerySettings {
    pub fn similarity_quality(&self) -> f32 {
        self.similarity_slider.similarity_quality.value()
    }
}
