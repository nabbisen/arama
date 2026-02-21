use iced::Task;

use super::{Footer, message::Message};

impl Footer {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {}
    }
}
