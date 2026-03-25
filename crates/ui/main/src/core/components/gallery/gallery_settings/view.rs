use iced::{
    Element,
    widget::{Button, button, row, text},
};

use super::{GallerySettings, message::Message};

impl GallerySettings {
    pub fn view(&self) -> Element<'_, Message> {
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
            button("🔍️").on_press_maybe(if self.embedding_cached {
                Some(Message::SimilarPairsOpen)
            } else {
                None
            });

        row![sub_dir_depth_limit, similar_pairs_button,]
            .spacing(20)
            .into()
    }
}
