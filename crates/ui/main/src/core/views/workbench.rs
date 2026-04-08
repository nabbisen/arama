use std::{collections::BTreeMap, path::PathBuf};

use iced::wgpu::naga::FastHashMap;

pub mod message;
mod update;
mod view;

const SPACING: u16 = 10;

pub struct Workbench {
    dir_path_thumbnail_path_map: BTreeMap<PathBuf, FastHashMap<String, String>>,
}

impl Workbench {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            dir_path_thumbnail_path_map: BTreeMap::default(),
        })
    }

    pub fn set_dir_path_thumbnail_path_map(
        &mut self,
        value: BTreeMap<PathBuf, FastHashMap<String, String>>,
    ) {
        self.dir_path_thumbnail_path_map = value;
    }

    pub fn embedding_cached(&mut self) -> bool {
        let embedding_cached = self
            .dir_path_thumbnail_path_map
            .iter()
            .any(|x| 1 < x.1.len());

        embedding_cached
    }
}
