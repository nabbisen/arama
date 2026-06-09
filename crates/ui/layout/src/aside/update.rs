use iced::Task;

use super::{
    Aside,
    message::{Event, Internal, Message},
};
use arama_ui_widgets::dir_tree;

impl Aside {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Event(_) => Task::none(),
            Message::Internal(Internal::DirTreeMessage(message)) => {
                let task = self
                    .dir_tree
                    .update(message.clone())
                    .map(|x| Message::Internal(Internal::DirTreeMessage(x)));
                match message {
                    dir_tree::message::Message::Event(message) => match message {
                        dir_tree::message::Event::DirClick(path) => {
                            Task::done(Message::Event(Event::DirSelect(path)))
                        }
                    },
                    dir_tree::message::Message::Internal(_) => task,
                }
            }
        }
    }
}
