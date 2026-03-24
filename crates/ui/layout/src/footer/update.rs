use iced::Task;

use super::{Footer, message::Message};

impl Footer {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ThumbnailSizeSliderMessage(message) => {
                let _ = self.thumbnail_size_slider.update(message);
            }
        }
        Task::none()
    }
}
