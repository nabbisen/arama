use arama_env::target_media_type::TargetMediaType;
use iced::{
    Element,
    widget::{Button, button, checkbox, row, text},
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

        let sub_dir_depth_limit = row![
            text("Sub dir depth"),
            button(text("⬇").size(12))
                .padding(2)
                .on_press_maybe(if 0 < self.sub_dir_depth_limit {
                    Some(Message::SubDirDepthLimitChanged(
                        self.sub_dir_depth_limit - 1,
                    ))
                } else {
                    None
                }),
            text(self.sub_dir_depth_limit),
            button(text("⬆").size(12))
                .padding(2)
                .on_press_maybe(if self.sub_dir_depth_limit < 2 {
                    Some(Message::SubDirDepthLimitChanged(
                        self.sub_dir_depth_limit + 1,
                    ))
                } else {
                    None
                }),
        ]
        .spacing(5);

        let similar_pairs_button: Button<Message> =
            button("Similar Pairs").on_press_maybe(if self.embedding_cached {
                Some(Message::SimilarPairsOpen)
            } else {
                None
            });

        row![
            media_types,
            sub_dir_depth_limit,
            self.thumbnail_size
                .view()
                .map(Message::ThumbnailSizeSliderMessage),
            similar_pairs_button,
        ]
        .spacing(20)
        .into()
    }
}
