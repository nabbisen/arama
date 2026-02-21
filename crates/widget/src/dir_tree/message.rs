use std::path::PathBuf;

use super::file_node;

#[derive(Debug, Clone)]
pub enum Message {
    FileNodeMessage(file_node::message::Message),
    DirClick(PathBuf),
    DirDoubleClick(PathBuf),
}
