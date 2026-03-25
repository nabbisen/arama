use std::path::PathBuf;

use super::{dir_nav, settings_nav};

#[derive(Debug, Clone)]
pub enum Message {
    DirSelect(PathBuf),
    SimilarPairsDialogOpen,
    SettingsOpen,
    DirNavMessage(dir_nav::message::Message),
    SettingsNavMessage(settings_nav::message::Message),
}
