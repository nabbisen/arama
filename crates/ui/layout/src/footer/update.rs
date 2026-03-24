use iced::Task;

use super::{Footer, message::Message};

impl Footer {
    pub fn update(&mut self, _message: Message) -> Task<Message> {
        Task::none()
    }
}
