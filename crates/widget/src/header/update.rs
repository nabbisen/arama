use iced::Task;

use super::{Header, message::Message};

impl Header {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::DirNavMessage(message) => {
                self.dir_nav.update(message).map(Message::DirNavMessage)
            }
            Message::SettingsMessage(message) => {
                self.settings.update(message).map(Message::SettingsMessage)
            }
        }
    }
}
