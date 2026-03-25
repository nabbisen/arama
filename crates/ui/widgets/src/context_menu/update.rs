use iced::Task;

use super::{ContextMenu, ContextMenuState, message::Message};

impl ContextMenu {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::FileManagerShow(path) => {
                let _ = file_handle::FileHandle::show(&path);
                self.state = ContextMenuState::None;
            }
        }
        Task::none()
    }
}
