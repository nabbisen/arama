use iced::Element;
use iced::widget::slider;

use crate::app::components::gallery::gallery_settings::similarity_slider::similarity_quality::SimilarityQuality;

use super::SimilaritySlider;
use super::message::Message;

impl SimilaritySlider {
    pub fn view(&self) -> Element<'_, Message> {
        slider(
            0..=(SimilarityQuality::len() - 1), // ← 離散レンジ
            self.similarity_quality.index(),    // ← enum → index
            Message::ValueChanged,
        )
        .step(1)
        .into()
    }
}
