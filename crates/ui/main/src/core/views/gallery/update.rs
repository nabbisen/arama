use iced::Task;

use super::{Gallery, message::Message};

impl Gallery {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ImageCellMessage(_message) => Task::none(),
        }
    }
}
