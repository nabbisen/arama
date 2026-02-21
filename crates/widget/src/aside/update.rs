use iced::Task;

use crate::aside::dir_tree;

use super::{Aside, message::Message};

impl Aside {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::DirTreeMessage(message) => {
                let task = self
                    .dir_tree
                    .update(message.clone())
                    .map(Message::DirTreeMessage);

                match message {
                    dir_tree::message::Message::DirClick(path) => {
                        return Task::done(Message::DirClick(path));
                    }
                    _ => (),
                }

                task
            }
            _ => Task::none(),
        }
    }
}
