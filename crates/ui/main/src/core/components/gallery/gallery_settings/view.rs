use iced::{
    Element,
    widget::{button, checkbox, row, text},
};

use super::{GallerySettings, message::Message};

impl GallerySettings {
    pub fn view(&self) -> Element<'_, Message> {
        let media_types = row![
            text("Image"),
            checkbox(self.media_type.include_image).on_toggle(Message::IncludeImage),
            text("Video"),
            checkbox(self.media_type.include_video).on_toggle(Message::IncludeVideo)
        ]
        .spacing(4);

        row![
            media_types,
            self.thumbnail_size
                .view()
                .map(Message::ThumbnailSizeSliderMessage),
            button("Similar Pairs").on_press(Message::SimilarPairsOpen)
        ]
        .spacing(10)
        .into()
    }
}
