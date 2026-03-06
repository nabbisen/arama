use std::{path::PathBuf, sync::Arc};

use arama_ai::{
    model::model_container::clip, pipeline::encode::image::embeddings::image_embedding,
};
use arama_cache::{CacheError, ImageCacheWriter, UpsertImageRequest};
use arama_env::cache_storage_path;
// use app_json_settings::ConfigManager;
// use arama_widget::dir_tree;
use iced::Task;
// use swdir::DirNode;

use crate::core::components::gallery::gallery_settings;

use super::{
    Gallery,
    message::Message,
    // subscription::Input
};
// use crate::core::settings::Settings;
// use arama_embedding::store::file::file_embedding_map::FileEmbeddingMap;

impl Gallery {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ImageCached(ret) => {
                let errors: Vec<_> = ret.iter().filter(|x| x.1.is_err()).collect();
                if 0 < errors.len() {
                    // todo error handling
                    eprintln!(
                        "{}",
                        errors
                            .into_iter()
                            .map(|x| format!("{:?}", x.1))
                            .collect::<Vec<String>>()
                            .join("\n")
                    );
                }

                if clip::model().ready().unwrap_or(false) {
                    Task::perform(
                        async {
                            image_embedding(ret.into_iter().map(|x| x.0).collect())
                                .await
                                .expect("failed to get embedding")
                        },
                        super::message::Message::EmbeddingCached,
                    )
                } else {
                    Task::none()
                }
            }
            Message::EmbeddingCached(err) => {
                if let Some(err) = err {
                    // todo error handling
                    eprintln!("{}", err);
                }
                self.gallery_settings.set_embedding_cached(true);
                Task::none()
            }
            // Message::EmbeddingCalculated(calculated) => {
            //     self.file_embedding_map = calculated.0;
            //     self.file_similar_pairs = calculated.1;
            //     Task::none()
            // }
            // Message::MenusMessage(message) => match message {
            //     menus::message::Message::ScaleUp => {
            //         if self.thumbnail_size <= 600 {
            //             self.thumbnail_size += 20;
            //         }
            //         Task::none()
            //     }
            //     menus::message::Message::ScaleDown => {
            //         if 40 <= self.thumbnail_size {
            //             self.thumbnail_size -= 20;
            //         }
            //         Task::none()
            //     }
            //     menus::message::Message::Quit => iced::exit(),
            // },
            Message::GallerySettingsMessage(message) => {
                let _ = self.gallery_settings.update(message.clone());

                match message {
                    gallery_settings::message::Message::SimilarPairsOpen => {
                        return Task::done(Message::SimilarPairsOpen);
                    }
                    //     gallery_settings::message::Message::SwdirDepthLimitMessage(message) => {
                    //         match message {
                    //             swdir_depth_limit::message::Message::ValueChanged(_) => {
                    //                 if let Some(dir_node) = self.dir_node.as_ref() {
                    //                     return Task::perform(
                    //                         super::util::load_images(
                    //                             dir_node.path.clone(),
                    //                             self.gallery_settings.swdir_depth_limit(),
                    //                         ),
                    //                         super::message::Message::ImagesLoaded,
                    //                     );
                    //                 }
                    //             }
                    //         }
                    //     }
                    _ => (),
                }

                Task::none()
            }
            Message::ImageCellMessage(_message) => Task::none(),
            Message::DirSelect(dir_node) => {
                self.dir_node = Some(dir_node.clone());
                Task::perform(
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
                    Message::ImageCached,
                )
            }
            Message::SimilarPairsOpen => Task::none(),
            // Message::ImageSelect(path) => {
            //     self.processing = true;
            //     self.selected_source_image = Some(path.clone());

            //     if let Some(tx) = &mut self.subscription_worker_tx {
            //         if let Some(dir_node) = self.dir_node.clone() {
            //             // let _ = tx.try_send(Input::ImageSimilarity((path, dir_node)));
            //             let _ = tx.try_send(Input::ImageSimilarity(dir_node));
            //         }
            //     }

            //     Task::none()
            // }
            // Message::SubscriptionWorkerReady(tx) => {
            //     self.subscription_worker_tx = Some(tx);

            //     if let Some(tx) = &mut self.subscription_worker_tx {
            //         // if let Some(path) = self.selected_source_image.clone() {
            //         if let Some(dir_node) = self.dir_node.clone() {
            //             // let _ = tx.try_send(Input::ImageSimilarity((path, dir_node)));
            //             let _ = tx.try_send(Input::ImageSimilarity(dir_node));
            //         }
            //         // }
            //     }

            //     Task::none()
            // }
            // Message::SubscriptionWorkerFinished(file_embedding) => {
            //     self.file_embedding_map.set_embedding(&file_embedding);
            //     self.processing = false;
            //     Task::none()
            // }
            // Message::SubscriptionWorkerFailed => {
            //     // error handling
            //     eprintln!("failed to calculate image similarity in background");
            //     self.file_embedding_map = FileEmbeddingMap::default();
            //     self.processing = false;
            //     Task::none()
            // }
            // Message::DirTreeMessage(message) => {
            //     let task = self.directory_tree.update(message.clone());

            //     match message {
            //         dir_tree::message::Message::DirectoryDoubleClick(path) => {
            //             ConfigManager::new()
            //                 .save(&Settings {
            //                     root_dir_path: path.to_string_lossy().into(),
            //                 })
            //                 .expect("failed to save config");

            //             self.clear();
            //             let dir_node = DirNode::with_path(path);
            //             self.dir_node = Some(dir_node.clone());

            //             return Task::perform(
            //                 super::util::load_images(
            //                     dir_node.path.clone(),
            //                     self.gallery_settings.swdir_depth_limit(),
            //                 ),
            //                 super::message::Message::ImagesLoaded,
            //             );
            //         }
            //         _ => (),
            //     }

            //     task.map(Message::DirTreeMessage)
            // }
        }
    }
}
