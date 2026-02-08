use iced::Task;
use swdir::DirNode;

use std::path::PathBuf;

use crate::app::{
    components::gallery::{menus::Menus, root_dir_select::RootDirSelect},
    utils::gallery::image_similarity::ImageSimilarity,
};

pub mod message;
mod update;
mod util;
mod view;

// アプリケーションの状態
pub struct Gallery {
    dir_node: Option<DirNode>,
    image_similarity: ImageSimilarity,
    thumbnail_size: u32,
    spacing: u32,
    menus: Menus,
    root_dir_select: RootDirSelect,
    selected_source_image: Option<PathBuf>,
    running: bool,
}

impl Gallery {
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
            image_similarity: ImageSimilarity::default(),
            thumbnail_size: 160, // サムネイルの正方形サイズ
            spacing: 10,         // 画像間の隙間
            menus: Menus::default(),
            root_dir_select: RootDirSelect::default(),
            selected_source_image: None,
            running: false,
        }
    }
}
