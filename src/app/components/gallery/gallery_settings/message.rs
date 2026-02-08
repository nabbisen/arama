use crate::app::components::gallery::gallery_settings::similarity_slider;

#[derive(Debug, Clone)]
pub enum Message {
    SimilaritySliderMessage(similarity_slider::message::Message),
}
