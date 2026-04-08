use iced::Task;

use super::{Workbench, message::Message};

impl Workbench {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ImageCellMessage(_message) => (),
            Message::CursorExit => (),
        }
        Task::none()
    }
}
