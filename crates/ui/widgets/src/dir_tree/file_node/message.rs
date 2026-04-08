use std::path::PathBuf;

use super::FileNode;

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
    ToggleExpand((PathBuf, bool, bool)),
    TreeLoaded(FileNode),
}
