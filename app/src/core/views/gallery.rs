// use iced::Task;
use swdir::DirNode;

use crate::core::{components::gallery::gallery_settings::GallerySettings, settings::Settings};

pub mod message;
// mod subscription;
mod update;
// mod util;
mod view;

const SPACING: u32 = 10;

// アプリケーションの状態
pub struct Gallery {
    dir_node: Option<DirNode>,
    gallery_settings: GallerySettings,
}

impl Gallery {
    pub fn new(settings: Option<&Settings>) -> Self {
        let path = if let Some(settings) = settings {
            &settings.root_dir_path
        } else {
            "."
        };

        Self {
            dir_node: Some(DirNode::with_path(path)),
            gallery_settings: GallerySettings::default(),
        }
    }

    // pub fn default_task(&self) -> Task<message::Message> {
    //     if let Some(dir_node) = &self.dir_node {
    //         Task::perform(
    //             util::load_images(dir_node.path.clone()),
    //             message::Message::ImagesLoaded,
    //         )
    //     } else {
    //         Task::none()
    //     }
    // }

    // fn clear(&mut self) {
    //     self.dir_node = None;
    // }
}
