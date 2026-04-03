use iced::Task;

use super::{Aside, message::Message};

impl Aside {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Open => self.is_open = true,
            Message::Close => self.is_open = false,
            Message::DirTreeMessage(message) => {
                let task = self.dir_tree.update(message.clone());
                return task.map(Message::DirTreeMessage);
            }
        }
        Task::none()
    }
}
