use iced::Task;

use super::file_node;

use super::{DirTree, message::Message};

impl DirTree {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::FileNodeMessage(file_node_message) => {
                let task = self.root.update(file_node_message.clone());

                match file_node_message {
                    file_node::message::Message::DirClick(path) => {
                        self.selected_path = Some(path.to_path_buf());
                        return Task::done(Message::DirClick(path));
                    }
                    _ => (),
                }

                return task.map(Message::FileNodeMessage);
            }
            Message::DirClick(_) => (),
        }
        Task::none()
    }
}
