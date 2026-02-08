use iced::Task;
use swdir::DirNode;

use std::path::PathBuf;

use crate::app::{
    components::gallery::{
        gallery_settings::GallerySettings, menus::Menus, root_dir_select::RootDirSelect,
    },
    utils::gallery::image_similarity::ImageSimilarity,
};

pub mod message;
mod update;
mod util;
mod view;

// アプリケーションの状態
pub struct Gallery {
    dir_node: Option<DirNode>,
    selected_source_image: Option<PathBuf>,
    running: bool,
    image_similarity: ImageSimilarity,
    thumbnail_size: u32,
    spacing: u32,
    menus: Menus,
    root_dir_select: RootDirSelect,
    gallery_settings: GallerySettings,
}

impl Gallery {
    pub fn new(root_dir_path: &str) -> Self {
        let mut ret: Gallery = Self::default();
        ret.dir_node = Some(DirNode::with_path(root_dir_path));
        ret
    }

    pub fn default_task(&self) -> Task<message::Message> {
        if let Some(dir_node) = &self.dir_node {
            Task::perform(
                util::load_images(dir_node.path.clone()),
                message::Message::ImagesLoaded,
            )
        } else {
            Task::none()
        }
    }

    fn clear(&mut self) {
        self.dir_node = None;
        self.image_similarity = ImageSimilarity::default();
        self.selected_source_image = None;
    }
}

impl Default for Gallery {
    fn default() -> Self {
        Self {
            // todo: load from config if saved
            dir_node: None,
            selected_source_image: None,
            running: false,
            image_similarity: ImageSimilarity::default(),
            thumbnail_size: 160, // サムネイルの正方形サイズ
            spacing: 10,         // 画像間の隙間
            menus: Menus::default(),
            root_dir_select: RootDirSelect::default(),
            gallery_settings: GallerySettings::default(),
        }
    }
}
