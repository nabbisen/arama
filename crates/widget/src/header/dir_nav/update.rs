use iced::Task;

use super::DirNav;
use super::message::Message;

impl DirNav {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {}
    }
}
