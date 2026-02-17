use super::similarity_slider::similarity_quality::SimilarityQuality;

pub mod message;
pub mod similarity_quality;
pub mod update;
pub mod view;

#[derive(Default)]
pub struct SimilaritySlider {
    pub similarity_quality: SimilarityQuality,
}
