use iced::Task;

use super::file_node;

use super::{
    DirTree,
    message::{Event, Internal, Message},
};

impl DirTree {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Event(_) => Task::none(),
            Message::Internal(message) => match message {
                Internal::FileNodeMessage(message) => {
                    let task = self
                        .root
                        .update(message.clone())
                        .map(|x| Message::Internal(Internal::FileNodeMessage(x)));

                    match message {
                        file_node::message::Message::Event(message) => match message {
                            file_node::message::Event::DirClick(path) => {
                                self.selected_path = Some(path.to_path_buf());
                                return Task::done(Message::Event(Event::DirClick(path)));
                            }
                        },
                        file_node::message::Message::Internal(_) => task,
                    }
                }
            },
        }
    }
}
