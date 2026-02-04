use std::path::PathBuf;

use iced::Task;

pub mod message;
mod update;
mod util;
mod view;

// アプリケーションの状態
pub struct Gallery {
    pub image_paths: Vec<PathBuf>,
    thumbnail_size: u32,
    spacing: u32,
}

impl Gallery {
    pub fn default_task() -> Task<message::Message> {
        Task::perform(util::load_images("."), message::Message::ImagesLoaded)
    }
}

impl Default for Gallery {
    fn default() -> Self {
        Self {
            image_paths: Vec::new(),
            thumbnail_size: 160, // サムネイルの正方形サイズ
            spacing: 10,         // 画像間の隙間
        }
    }
}
