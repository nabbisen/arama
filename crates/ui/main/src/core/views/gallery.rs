use std::{collections::BTreeMap, path::PathBuf};

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
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            dir_path_thumbnail_path_map: BTreeMap::default(),
            gallery_settings: GallerySettings::new(),
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
}
