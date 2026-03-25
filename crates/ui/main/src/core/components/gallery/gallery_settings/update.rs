use iced::Task;

use super::{GallerySettings, message::Message};

impl GallerySettings {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SubDirDepthLimitChanged(value) => self.sub_dir_depth_limit = value,
            Message::SimilarPairsOpen => (),
        }
        Task::none()
    }
}
