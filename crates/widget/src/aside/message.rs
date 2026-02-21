use std::path::PathBuf;

use super::dir_tree;

#[derive(Debug, Clone)]
pub enum Message {
    DirClick(PathBuf),
    DirTreeMessage(dir_tree::message::Message),
}
