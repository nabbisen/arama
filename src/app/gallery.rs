use std::path::PathBuf;

pub mod message;
mod update;
mod view;

// アプリケーションの状態
pub struct Gallery {
    pub image_paths: Vec<PathBuf>,
    thumbnail_size: u32,
    spacing: u32,
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
