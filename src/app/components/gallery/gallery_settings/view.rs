use iced::{Element, widget::row};

use super::{GallerySettings, message::Message};

impl GallerySettings {
    pub fn view(&self) -> Element<'_, Message> {
        row![
            self.similarity_slider.similarity_quality.label(),
            self.similarity_slider
                .view()
                .map(Message::SimilaritySliderMessage)
        ]
        .into()
    }
}
