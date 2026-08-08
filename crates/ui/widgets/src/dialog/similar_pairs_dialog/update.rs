use iced::Task;

use super::{SimilarPairsDialog, message::Message};

impl SimilarPairsDialog {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::EmbeddingsReady(outcome) => {
                self.has_read_error = outcome.had_errors;
                self.pairs = Some(outcome.items);
            }
            Message::MediaItemEnter(path) => self.hovered_media_item_path_str = Some(path),
            Message::MediaItemDoubleClicked(_) => (),
            Message::MediaExit => self.hovered_media_item_path_str = None,
        }
        Task::none()
    }
}
