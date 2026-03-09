use iced::Task;

use super::{SimilarPairsDialog, message::Message};

impl SimilarPairsDialog {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::EmbeddingsReady(pairs) => {
                self.pairs = Some(pairs);
                Task::none()
            }
        }
    }
}
