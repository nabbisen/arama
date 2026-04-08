use std::path::PathBuf;

use super::{dir_nav, settings_nav};

#[derive(Debug, Clone)]
pub enum Message {
    Event(Event),
    Internal(Internal),
}

#[derive(Debug, Clone)]
pub enum Event {
    DirSelect(PathBuf),
    SimilarPairsDialogOpen,
    SettingsOpen,
}

#[derive(Debug, Clone)]
pub enum Internal {
    DirNavMessage(dir_nav::message::Message),
    SettingsNavMessage(settings_nav::message::Message),
}
