use std::path::PathBuf;

use iced::Task;

use crate::app::components::gallery::{menus::Menus, root_dir_select::RootDirSelect};

pub mod message;
mod update;
mod util;
mod view;

// アプリケーションの状態
pub struct Gallery {
    root_dir: PathBuf,
    image_paths: Vec<(PathBuf, Option<f32>)>,
    thumbnail_size: u32,
    spacing: u32,
    menus: Menus,
    root_dir_select: RootDirSelect,
    selected_source_image: Option<PathBuf>,
}

impl Gallery {
    pub fn default_task(&self) -> Task<message::Message> {
        Task::perform(
            util::load_images(self.root_dir.clone()),
            message::Message::ImagesLoaded,
        )
    }
}

impl Default for Gallery {
    fn default() -> Self {
        Self {
            root_dir: PathBuf::from("."),
            image_paths: Vec::new(),
            thumbnail_size: 160, // サムネイルの正方形サイズ
            spacing: 10,         // 画像間の隙間
            menus: Menus::default(),
            root_dir_select: RootDirSelect::default(),
            selected_source_image: None,
        }
    }
}
