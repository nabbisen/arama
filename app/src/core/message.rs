use arama_widget::dir_tree;

use super::gallery;
use crate::core::components::common::model_loader;

pub enum Message {
    GalleryMessage(gallery::message::Message),
    DirTreeMessage(dir_tree::message::Message),
    ModelLoaderMessage(model_loader::Message),
}
