use super::file_node;

use super::{DirTree, message::Message, output::Output};

impl DirTree {
    pub fn update(&mut self, message: Message) -> Option<Output> {
        match message {
            Message::FileNodeMessage(file_node_message) => {
                let _ = self.root.update(file_node_message.clone());

                match file_node_message {
                    file_node::message::Message::DirClick(path) => {
                        self.selected_path = Some(path.to_path_buf());
                        return Some(Output::DirClick(path));
                    }
                    _ => (),
                }
            }
            _ => (),
        }
        None
    }
}
