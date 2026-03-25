use iced::Task;

use super::{GallerySettings, message::Message};

impl GallerySettings {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SimilarPairsOpen => (),
        }
        Task::none()
    }
}
