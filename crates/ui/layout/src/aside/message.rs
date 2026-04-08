use std::path::PathBuf;

use arama_ui_widgets::dir_tree;

#[derive(Debug, Clone)]
pub enum Message {
    Event(Event),
    Internal(Internal),
}

#[derive(Debug, Clone)]
pub enum Event {
    DirSelect(PathBuf),
}

#[derive(Debug, Clone)]
pub enum Internal {
    Open,
    Close,
    DirTreeMessage(dir_tree::message::Message),
}
