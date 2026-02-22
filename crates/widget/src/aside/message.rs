use super::dir_tree;

#[derive(Debug, Clone)]
pub enum Message {
    DirTreeMessage(dir_tree::message::Message),
}
