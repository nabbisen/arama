use app_json_settings::ConfigManager;
use iced::Task;
use swdir::DirNode;

use super::{Gallery, message::Message, subscription::Input};
use crate::{
    app::{
        components::gallery::{
            gallery_settings::{self, swdir_depth_limit},
            menus, root_dir_select,
        },
        settings::Settings,
    },
    engine::store::file::file_embedding_map::FileEmbeddingMap,
};

impl Gallery {
    // アプリケーション初期化時に画像を読み込むTaskを発行
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ImagesLoaded(dir_node) => {
                self.dir_node = Some(dir_node.clone());
                Task::perform(
                    super::util::calculate_embedding(
                        dir_node,
                        self.gallery_settings.similarity_quality(),
                    ),
                    super::message::Message::EmbeddingCalculated,
                )
            }
            Message::EmbeddingCalculated(calculated) => {
                self.file_embedding_map = calculated.0;
                self.file_similar_pairs = calculated.1;
                Task::none()
            }
            Message::MenusMessage(message) => match message {
                menus::message::Message::ScaleUp => {
                    if self.thumbnail_size <= 600 {
                        self.thumbnail_size += 20;
                    }
                    Task::none()
                }
                menus::message::Message::ScaleDown => {
                    if 40 <= self.thumbnail_size {
                        self.thumbnail_size -= 20;
                    }
                    Task::none()
                }
                menus::message::Message::Quit => iced::exit(),
            },
            Message::GallerySettingsMessage(message) => {
                let _ = self.gallery_settings.update(message.clone());

                match message {
                    gallery_settings::message::Message::SwdirDepthLimitMessage(message) => {
                        match message {
                            swdir_depth_limit::message::Message::ValueChanged(_) => {
                                if let Some(dir_node) = self.dir_node.as_ref() {
                                    return Task::perform(
                                        super::util::load_images(
                                            dir_node.path.clone(),
                                            self.gallery_settings.swdir_depth_limit(),
                                        ),
                                        super::message::Message::ImagesLoaded,
                                    );
                                }
                            }
                        }
                    }
                    _ => (),
                }

                Task::none()
            }
            Message::RootDirSelectMessage(message) => {
                let task = self
                    .root_dir_select
                    .update(message.clone())
                    .map(|message| Message::RootDirSelectMessage(message));

                match message {
                    root_dir_select::message::Message::DialogClose(path) => {
                        if let Some(path) = path {
                            ConfigManager::new()
                                .save(&Settings {
                                    root_dir_path: path.to_string_lossy().into(),
                                })
                                .expect("failed to save config");

                            self.clear();
                            let dir_node = DirNode::with_path(path);
                            self.dir_node = Some(dir_node.clone());

                            return Task::perform(
                                super::util::load_images(
                                    dir_node.path.clone(),
                                    self.gallery_settings.swdir_depth_limit(),
                                ),
                                super::message::Message::ImagesLoaded,
                            );
                        }
                    }
                    _ => (),
                }

                task
            }
            Message::ImageSelect(path) => {
                self.processing = true;
                self.selected_source_image = Some(path.clone());

                if let Some(tx) = &mut self.subscription_worker_tx {
                    if let Some(dir_node) = self.dir_node.clone() {
                        // let _ = tx.try_send(Input::ImageSimilarity((path, dir_node)));
                        let _ = tx.try_send(Input::ImageSimilarity(dir_node));
                    }
                }

                Task::none()
            }
            Message::SubscriptionWorkerReady(tx) => {
                self.subscription_worker_tx = Some(tx);

                if let Some(tx) = &mut self.subscription_worker_tx {
                    // if let Some(path) = self.selected_source_image.clone() {
                    if let Some(dir_node) = self.dir_node.clone() {
                        // let _ = tx.try_send(Input::ImageSimilarity((path, dir_node)));
                        let _ = tx.try_send(Input::ImageSimilarity(dir_node));
                    }
                    // }
                }

                Task::none()
            }
            Message::SubscriptionWorkerFinished(file_embedding) => {
                self.file_embedding_map.set_embedding(&file_embedding);
                self.processing = false;
                Task::none()
            }
            Message::SubscriptionWorkerFailed => {
                // error handling
                eprintln!("failed to calculate image similarity in background");
                self.file_embedding_map = FileEmbeddingMap::default();
                self.processing = false;
                Task::none()
            }
        }
    }
}
