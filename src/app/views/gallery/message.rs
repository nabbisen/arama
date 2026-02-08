use std::path::PathBuf;

use swdir::DirNode;

use crate::app::{
    components::gallery::{gallery_settings, menus, root_dir_select},
    utils::gallery::image_similarity::ImageSimilarity,
};

#[derive(Debug, Clone)]
pub enum Message {
    ImagesLoaded(DirNode),
    MenusMessage(menus::message::Message),
    RootDirSelectMessage(root_dir_select::message::Message),
    ImageSelect(PathBuf),
    ImageSimilarityCompleted(ImageSimilarity),
    GallerySettingsMessage(gallery_settings::message::Message),
}
