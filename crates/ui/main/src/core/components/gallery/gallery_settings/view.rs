use arama_env::target_media_type::TargetMediaType;
use iced::{
    Element,
    widget::{Button, button, checkbox, row},
};

use super::{GallerySettings, message::Message};

impl GallerySettings {
    pub fn view(&self) -> Element<'_, Message> {
        let media_types = row![
            checkbox(self.target_media_type.include_image)
                .label("Image")
                .on_toggle(|x| {
                    Message::TargetMediaTypeChanged(TargetMediaType {
                        include_image: x,
                        include_video: self.target_media_type.include_video,
                    })
                }),
            checkbox(self.target_media_type.include_video)
                .label("Video")
                .on_toggle(|x| {
                    Message::TargetMediaTypeChanged(TargetMediaType {
                        include_image: self.target_media_type.include_image,
                        include_video: x,
                    })
                })
        ]
        .spacing(10);

        let similar_pairs_button: Button<Message> =
            button("Similar Pairs").on_press_maybe(if self.embedding_cached {
                Some(Message::SimilarPairsOpen)
            } else {
                None
            });

        row![
            media_types,
            self.thumbnail_size
                .view()
                .map(Message::ThumbnailSizeSliderMessage),
            similar_pairs_button,
        ]
        .spacing(20)
        .into()
    }
}
