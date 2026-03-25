use iced::{
    Element,
    widget::{Button, button, row},
};

use super::{GallerySettings, message::Message};

impl GallerySettings {
    pub fn view(&self) -> Element<'_, Message> {
        let similar_pairs_button: Button<Message> =
            button("🔍️").on_press_maybe(if self.embedding_cached {
                Some(Message::SimilarPairsOpen)
            } else {
                None
            });

        row![similar_pairs_button,].spacing(20).into()
    }
}
