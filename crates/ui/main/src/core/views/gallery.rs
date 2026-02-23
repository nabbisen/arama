use std::path::PathBuf;

use arama_cache::CacheProducer;
use arama_env::{IMAGE_EXTENSION_ALLOWLIST, MAX_THUMBNAIL_SIZE, VIDEO_EXTENSION_ALLOWLIST};
use iced::Task;
// use iced::Task;
use swdir::{DirNode, Swdir};

use crate::{
    components::gallery::gallery_settings::target_media_type::TargetMediaType,
    core::components::gallery::gallery_settings::GallerySettings,
};

pub mod message;
// mod subscription;
mod update;
mod util;
mod view;

const SPACING: u16 = 10;

// アプリケーションの状態
pub struct Gallery {
    dir_node: Option<DirNode>,
    pub gallery_settings: GallerySettings,
    cache_producer: CacheProducer,
}

impl Gallery {
    pub fn new<T: Into<PathBuf>>(
        root_dir_path: T,
        target_media_type: &TargetMediaType,
    ) -> anyhow::Result<Self> {
        let mut extension_allowlist: Vec<&str> = vec![];
        if target_media_type.include_image {
            extension_allowlist.extend(IMAGE_EXTENSION_ALLOWLIST);
        }
        if target_media_type.include_video {
            extension_allowlist.extend(VIDEO_EXTENSION_ALLOWLIST);
        }

        let dir_node = Swdir::default()
            .set_root_path(root_dir_path)
            .set_extension_allowlist(&extension_allowlist)?
            .walk();

        Ok(Self {
            dir_node: Some(dir_node),
            gallery_settings: GallerySettings::default(),
            cache_producer: CacheProducer::new(
                MAX_THUMBNAIL_SIZE as u32,
                MAX_THUMBNAIL_SIZE as u32,
            )?,
        })
    }

    pub fn default_task(&self) -> Task<message::Message> {
        if let Some(dir_node) = self.dir_node.as_ref() {
            Task::perform(
                self.cache_producer.clone().refresh(dir_node.clone()),
                message::Message::ImageCached,
            )
        } else {
            Task::none()
        }
    }

    pub fn dir_node(&self) -> Option<DirNode> {
        self.dir_node.clone()
    }

    pub fn thumbnail_size(&self) -> u16 {
        self.gallery_settings.thumbnail_size()
    }

    // fn clear(&mut self) {
    //     self.dir_node = None;
    // }
}
