use arama_widget::directory_tree::DirectoryTree;
use iced::{Task, futures::channel::mpsc::Sender};
use swdir::DirNode;

use std::path::PathBuf;

use crate::core::components::gallery::{gallery_settings::GallerySettings, menus::Menus};
use arama_embedding::store::file::file_embedding_map::FileEmbeddingMap;
use subscription::Input;

pub mod message;
mod subscription;
mod update;
mod util;
mod view;

// アプリケーションの状態
pub struct Gallery {
    dir_node: Option<DirNode>,
    selected_source_image: Option<PathBuf>,
    processing: bool,
    file_embedding_map: FileEmbeddingMap,
    file_similar_pairs: Vec<(PathBuf, PathBuf, f32)>,
    thumbnail_size: u32,
    spacing: u32,
    menus: Menus,
    gallery_settings: GallerySettings,
    subscription_worker_tx: Option<Sender<Input>>,
    directory_tree: DirectoryTree,
}

impl Gallery {
    pub fn new(root_dir_path: &str) -> Self {
        let mut ret: Gallery = Self::default();
        ret.dir_node = Some(DirNode::with_path(root_dir_path));
        ret
    }

    pub fn default_task(&self) -> Task<message::Message> {
        if let Some(dir_node) = &self.dir_node {
            Task::perform(
                util::load_images(
                    dir_node.path.clone(),
                    self.gallery_settings.swdir_depth_limit(),
                ),
                message::Message::ImagesLoaded,
            )
        } else {
            Task::none()
        }
    }

    fn clear(&mut self) {
        self.dir_node = None;
        self.file_embedding_map = FileEmbeddingMap::default();
        self.selected_source_image = None;
    }
}

impl Default for Gallery {
    fn default() -> Self {
        Self {
            // todo: load from config if saved
            dir_node: None,
            selected_source_image: None,
            processing: false,
            file_embedding_map: FileEmbeddingMap::default(),
            file_similar_pairs: vec![],
            thumbnail_size: 160, // サムネイルの正方形サイズ
            spacing: 10,         // 画像間の隙間
            menus: Menus::default(),
            gallery_settings: GallerySettings::default(),
            subscription_worker_tx: None,
            directory_tree: DirectoryTree::new(".", false, false),
        }
    }
}
