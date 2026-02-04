use std::path::PathBuf;

use crate::app::components::gallery::{menus, root_dir_select};

#[derive(Debug, Clone)]
pub enum Message {
    ImagesLoaded(Vec<PathBuf>),
    MenusMessage(menus::message::Message),
    RootDirSelectMessage(root_dir_select::message::Message),
    ImageSelect(PathBuf),
    ImageSimilarityCompleted(Vec<(PathBuf, Option<f32>)>),
}
