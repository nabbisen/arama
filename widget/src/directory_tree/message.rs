use crate::directory_tree::file_node;

#[derive(Debug, Clone)]
pub enum Message {
    FileNodeMessage(file_node::message::Message),
}
