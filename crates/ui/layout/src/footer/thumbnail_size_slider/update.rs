use iced::Task;

use super::{ThumbnailSizeSlider, message::Message};

impl ThumbnailSizeSlider {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ValueChanged(value) => {
                self.value = value;
            }
        }
        Task::none()
    }
}
