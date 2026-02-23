use iced::Task;

use super::{FileSystemSettings, message::Message};

impl FileSystemSettings {
    pub fn update(&mut self, _message: Message) -> Task<Message> {
        // match message {
        //     _ => Task::none(),
        // }
        Task::none()
    }
}
