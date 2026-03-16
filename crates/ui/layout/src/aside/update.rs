use iced::Task;

use super::{Aside, message::Message};

impl Aside {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::DirTreeMessage(message) => {
                let task = self.dir_tree.update(message.clone());
                task.map(Message::DirTreeMessage)
            }
        }
    }
}
