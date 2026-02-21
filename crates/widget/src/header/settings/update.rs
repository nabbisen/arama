use iced::Task;

use super::Settings;
use super::message::Message;

impl Settings {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {}
    }
}
