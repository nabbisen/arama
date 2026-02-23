use iced::{
    Element,
    widget::{button, row},
};

use super::{GallerySettings, message::Message};

impl GallerySettings {
    pub fn view(&self) -> Element<'_, Message> {
        row![
            self.thumbnail_size
                .view()
                .map(Message::ThumbnailSizeSliderMessage),
            button("Similar Pairs").on_press(Message::SimilarPairsOpen)
        ]
        .into()
    }
}
