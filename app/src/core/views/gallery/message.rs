// use std::path::PathBuf;

// use arama_widget::dir_tree;
// use iced::futures::channel::mpsc::Sender;
// use swdir::DirNode;

use swdir::DirNode;

// use super::subscription::Input;
use crate::core::components::gallery::gallery_settings;

#[derive(Debug, Clone)]
pub enum Message {
    ImageCached(Vec<String>),
    EmbeddingCached(Option<String>),
    // EmbeddingCalculated((FileEmbeddingMap, Vec<(PathBuf, PathBuf, f32)>)),
    // MenusMessage(menus::message::Message),
    // ImageSelect(PathBuf),
    GallerySettingsMessage(gallery_settings::message::Message),
    // SubscriptionWorkerReady(Sender<Input>),
    // SubscriptionWorkerFinished(FileEmbedding),
    // SubscriptionWorkerFailed,
    // DirTreeMessage(dir_tree::message::Message),
    DirSelect(DirNode),
}
