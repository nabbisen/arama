use iced::Task;

use super::{GeneralSettings, message::Message};

impl GeneralSettings {
    pub fn update(&mut self, _message: Message) -> Task<Message> {
        // match message {
        //     _ => Task::none(),
        // }
        Task::none()
    }
}
