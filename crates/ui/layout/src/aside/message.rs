use std::path::PathBuf;

use iced_swdir_tree::DirectoryTreeEvent;

#[derive(Debug, Clone)]
pub enum Message {
    Event(Event),
    Internal(Internal),
}

/// Events that bubble up to the parent application.
#[derive(Debug, Clone)]
pub enum Event {
    DirSelect(PathBuf),
}

/// Internal routing: async scan results and drag machinery from the tree widget.
#[derive(Debug, Clone)]
pub enum Internal {
    TreeEvent(DirectoryTreeEvent),
    /// Emitted when the expand cascade finishes: scroll the tree into view.
    ExpandDone,
}
