use iced::Task;

use super::{ContextMenu, ContextMenuState, message::Message};

impl ContextMenu {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::OpenWithDefault(path) => {
                let _ = file_handle::FileHandle::open_with_default(&path);
                self.state = ContextMenuState::None;
            }
            Message::FileManagerShow(path) => {
                let _ = file_handle::FileHandle::show(&path);
                self.state = ContextMenuState::None;
            }
        }
        Task::none()
    }
}
