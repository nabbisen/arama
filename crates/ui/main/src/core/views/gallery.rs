use std::{collections::BTreeMap, path::PathBuf};

use arama_env::{
    IMAGE_EXTENSION_ALLOWLIST, VIDEO_EXTENSION_ALLOWLIST, target_media_type::TargetMediaType,
};
use iced::wgpu::naga::FastHashMap;

use crate::core::components::gallery::gallery_settings::GallerySettings;

pub mod message;
mod update;
mod view;

const SPACING: u16 = 10;

// アプリケーションの状態
pub struct Gallery {
    dir_path_thumbnail_path_map: BTreeMap<PathBuf, FastHashMap<String, String>>,
    pub gallery_settings: GallerySettings,
}

impl Gallery {
    pub fn new(
        target_media_type: &TargetMediaType,
        sub_dir_depth_limit: u8,
    ) -> anyhow::Result<Self> {
        let mut extension_allowlist: Vec<&str> = vec![];
        if target_media_type.include_image {
            extension_allowlist.extend(IMAGE_EXTENSION_ALLOWLIST);
        }
        if target_media_type.include_video {
            extension_allowlist.extend(VIDEO_EXTENSION_ALLOWLIST);
        }

        Ok(Self {
            dir_path_thumbnail_path_map: BTreeMap::default(),
            gallery_settings: GallerySettings::new(target_media_type, sub_dir_depth_limit),
        })
    }

    pub fn set_dir_path_thumbnail_path_map(
        &mut self,
        value: BTreeMap<PathBuf, FastHashMap<String, String>>,
    ) {
        self.dir_path_thumbnail_path_map = value;
    }

    pub fn update_embedding_cached(&mut self) {
        let embedding_cached = self
            .dir_path_thumbnail_path_map
            .iter()
            .any(|x| 1 < x.1.len());

        self.gallery_settings.set_embedding_cached(embedding_cached);
    }

    pub fn thumbnail_size(&self) -> u16 {
        self.gallery_settings.thumbnail_size()
    }
}
