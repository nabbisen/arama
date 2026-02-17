use std::path::PathBuf;

use iced::futures::channel::mpsc::Sender;
use swdir::DirNode;

use super::subscription::Input;
use crate::{
    app::components::gallery::{gallery_settings, menus, root_dir_select},
    engine::store::file::{file_embedding::FileEmbedding, file_embedding_map::FileEmbeddingMap},
};

#[derive(Debug, Clone)]
pub enum Message {
    ImagesLoaded(DirNode),
    EmbeddingCalculated((FileEmbeddingMap, Vec<(PathBuf, PathBuf, f32)>)),
    MenusMessage(menus::message::Message),
    RootDirSelectMessage(root_dir_select::message::Message),
    ImageSelect(PathBuf),
    GallerySettingsMessage(gallery_settings::message::Message),
    SubscriptionWorkerReady(Sender<Input>),
    SubscriptionWorkerFinished(FileEmbedding),
    SubscriptionWorkerFailed,
}
