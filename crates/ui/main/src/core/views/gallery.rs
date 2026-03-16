use std::{path::PathBuf, sync::Arc};

use arama_cache::{ImageCacheWriter, UpsertImageRequest};
use arama_env::{
    IMAGE_EXTENSION_ALLOWLIST, MAX_THUMBNAIL_SIZE, VIDEO_EXTENSION_ALLOWLIST, cache_storage_path,
    target_media_type::TargetMediaType,
};
use iced::{Task, wgpu::naga::FastHashMap};
// use iced::Task;
use swdir::{DirNode, Swdir};

use crate::core::components::gallery::gallery_settings::GallerySettings;

pub mod message;
// mod subscription;
mod update;
mod view;

const SPACING: u16 = 10;

// アプリケーションの状態
pub struct Gallery {
    dir_node: Option<DirNode>,
    path_thumbnail_path_map: FastHashMap<String, String>,
    pub gallery_settings: GallerySettings,
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
            path_thumbnail_path_map: FastHashMap::default(),
            gallery_settings: GallerySettings::new(target_media_type),
        })
    }

    pub fn default_task(&self) -> Task<message::Message> {
        if let Some(dir_node) = self.dir_node.as_ref() {
            let dir_node = dir_node.clone();
            Task::perform(
                // self.cache_producer.clone().refresh(dir_node.clone()),
                async move {
                    let writer = ImageCacheWriter::onetime(arama_cache::DbLocation::Custom(
                        cache_storage_path().expect("failed to get cache stogate path"),
                    ))
                    // todo: error handling
                    .expect("failed to get cache writer");
                    let requests: Vec<UpsertImageRequest> = dir_node
                        .flatten_paths()
                        .iter()
                        .map(|x| UpsertImageRequest {
                            path: x.to_path_buf(),
                            clip_vector: None,
                        })
                        .collect();
                    let ret = writer.upsert_all(requests);
                    ret.into_iter()
                        .map(|x| (x.0, Arc::new(x.1)))
                        .collect::<Vec<(PathBuf, Arc<arama_cache::Result<()>>)>>()
                },
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
