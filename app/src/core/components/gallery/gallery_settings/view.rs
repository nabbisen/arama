use iced::Element;

use super::{GallerySettings, message::Message};

impl GallerySettings {
    pub fn view(&self) -> Element<'_, Message> {
        self.thumbnail_size
            .view()
            .map(Message::ThumbnailSizeSliderMessage)
            .into()
    }
}
