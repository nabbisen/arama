use arama_indexer::ImageCacheManager;
use iced::Task;
// use iced::Task;
use swdir::{DirNode, Swdir};

use crate::core::{
    components::gallery::gallery_settings::{
        GallerySettings, thumbnail_size_slider::MAX_THUMBNAIL_SIZE,
    },
    settings::Settings,
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
    pub fn new(settings: Option<&Settings>) -> anyhow::Result<Self> {
        let path = if let Some(settings) = settings {
            &settings.root_dir_path
        } else {
            "."
        };

        let dir_node = Swdir::default()
            .set_root_path(path)
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
                util::image_cache(dir_node.clone(), self.image_cache_manager.clone()),
                message::Message::ImageCached,
            )
        } else {
            Task::none()
        }
    }

    // fn clear(&mut self) {
    //     self.dir_node = None;
    // }
}
