use std::path::PathBuf;

use super::file_node;

#[derive(Debug, Clone)]
pub enum Message {
    Event(Event),
    Internal(Internal),
}

#[derive(Debug, Clone)]
pub enum Event {
    DirClick(PathBuf),
}

#[derive(Debug, Clone)]
pub enum Internal {
    FileNodeMessage(file_node::message::Message),
}
