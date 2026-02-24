use iced::{
    Element,
    widget::{Button, button, checkbox, row, text},
};

use super::{GallerySettings, message::Message};

impl GallerySettings {
    pub fn view(&self) -> Element<'_, Message> {
        let media_types = row![
            text("Image"),
            checkbox(self.target_media_type.include_image).on_toggle(Message::IncludeImage),
            text("Video"),
            checkbox(self.target_media_type.include_video).on_toggle(Message::IncludeVideo)
        ]
        .spacing(4);

        let mut similar_pairs_button: Button<Message> = button("Similar Pairs");
        if self.embedding_cached {
            similar_pairs_button = similar_pairs_button.on_press(Message::SimilarPairsOpen);
        }

        row![
            media_types,
            self.thumbnail_size
                .view()
                .map(Message::ThumbnailSizeSliderMessage),
            similar_pairs_button,
        ]
        .spacing(10)
        .into()
    }
}
