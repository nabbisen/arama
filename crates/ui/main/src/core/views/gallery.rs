use std::path::PathBuf;

use arama_cache::ImageCacheManager;
use iced::Task;
// use iced::Task;
use swdir::{DirNode, Swdir};

use crate::core::components::gallery::gallery_settings::{
    GallerySettings, thumbnail_size_slider::MAX_THUMBNAIL_SIZE,
};

pub mod message;
// mod subscription;
mod update;
mod util;
mod view;

pub const EXTENSION_ALLOWLIST: &[&str; 6] = &["png", "jpg", "jpeg", "webp", "gif", "bmp"];
const SPACING: u16 = 10;

// アプリケーションの状態
pub struct Gallery {
    dir_node: Option<DirNode>,
    gallery_settings: GallerySettings,
    image_cache_manager: ImageCacheManager,
}

impl Gallery {
    pub fn new<T: Into<PathBuf>>(root_dir_path: T) -> anyhow::Result<Self> {
        let dir_node = Swdir::default()
            .set_root_path(root_dir_path)
            .set_extension_allowlist(EXTENSION_ALLOWLIST)?
            .walk();

        Ok(Self {
            dir_node: Some(dir_node),
            gallery_settings: GallerySettings::default(),
            image_cache_manager: ImageCacheManager::new(
                MAX_THUMBNAIL_SIZE as u32,
                MAX_THUMBNAIL_SIZE as u32,
            )?,
        })
    }

    pub fn default_task(&self) -> Task<message::Message> {
        if let Some(dir_node) = self.dir_node.as_ref() {
            Task::perform(
                self.image_cache_manager.clone().refresh(dir_node.clone()),
                message::Message::ImageCached,
            )
        } else {
            Task::none()
        }
    }

    pub fn dir_node(&self) -> Option<DirNode> {
        self.dir_node.clone()
    }

    // fn clear(&mut self) {
    //     self.dir_node = None;
    // }
}
