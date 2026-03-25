use std::{collections::BTreeMap, path::PathBuf, sync::Arc};

use arama_ai::{
    model::model_container::clip, pipeline::encode::image::embeddings::image_embedding,
};
use arama_cache::{
    DbLocation, ImageCacheReader, ImageCacheWriter, LookupResult, UpsertImageRequest,
    VideoCacheReader,
};
use arama_env::{IMAGE_EXTENSION_ALLOWLIST, VIDEO_EXTENSION_ALLOWLIST, cache_storage_path};
use arama_ui_main::{components::gallery::image_cell, views::gallery};
use iced::{Task, wgpu::naga::FastHashMap};
use swdir::{DirNode, Recurse, Swdir};

use super::{App, Dialog, message::Message};
use arama_ui_layout::{aside, footer, header};
use arama_ui_widgets::{
    context_menu::ContextMenuState,
    dialog::{media_focus_dialog, settings_dialog, similar_pairs_dialog},
    dir_tree,
};

impl App {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::CacheRequire => {
                if let Some(dir_node) = &self.dir_node {
                    let dir_node = dir_node.clone();

                    Task::perform(
                        async move {
                            let writer =
                                ImageCacheWriter::onetime(arama_cache::DbLocation::Custom(
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
                        Message::ThumbnailCacheFinished,
                    )
                } else {
                    self.processing_off();
                    Task::none()
                }
            }
            Message::ThumbnailCacheFinished(ret) => {
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

                if let Some(dir_node) = &self.dir_node {
                    let image_cache_reader = ImageCacheReader::onetime(DbLocation::Custom(
                        cache_storage_path().expect("failed to get storaget path"),
                    ))
                    .expect("failed to get video cache reader");

                    let video_cache_reader = VideoCacheReader::onetime(DbLocation::Custom(
                        cache_storage_path().expect("failed to get storaget path"),
                    ))
                    .expect("failed to get video cache reader");

                    self.gallery
                        .set_dir_path_thumbnail_path_map(dir_path_thumbnail_path_map(
                            &dir_node,
                            &image_cache_reader,
                            &video_cache_reader,
                        ));

                    self.header
                        .set_embedding_cached(self.gallery.embedding_cached());
                }

                if clip::model().ready().unwrap_or(false) {
                    Task::perform(
                        async {
                            image_embedding(ret.into_iter().map(|x| x.0).collect())
                                .await
                                .expect("failed to get embedding")
                        },
                        Message::EmbeddingCacheFinished,
                    )
                } else {
                    self.processing_off();
                    Task::none()
                }
            }
            Message::EmbeddingCacheFinished(err) => {
                if let Some(err) = err {
                    // todo error handling
                    eprintln!("{}", err);
                }

                self.aside.set_processing(self.processing);
                self.header
                    .set_embedding_cached(self.gallery.embedding_cached());

                self.processing_off();
                Task::none()
            }
            Message::SetupMessage(message) => {
                let task = self
                    .setup
                    .update(message.clone())
                    .map(Message::SetupMessage);
                if self.setup.finished {
                    self.processing_on();
                    Task::done(Message::CacheRequire)
                } else {
                    task
                }
            }
            Message::GalleryMessage(message) => {
                let task = self
                    .gallery
                    .update(message.clone())
                    .map(Message::GalleryMessage);

                match message {
                    gallery::message::Message::ImageCellMessage(message) => match message {
                        image_cell::message::Message::ImageCellEnter(path) => {
                            self.image_cell_path_update(Some(path));
                        }
                        image_cell::message::Message::ImageSelect => {
                            if let Some(path) = &self.image_cell_path {
                                let media_focus_dialog =
                                    media_focus_dialog::MediaFocusDialog::new(path);
                                let dialog = Dialog::MediaFocusDialog(media_focus_dialog.clone());
                                let default_task = media_focus_dialog.default_task();

                                self.dialog = Some(dialog);

                                return Task::batch([
                                    task,
                                    default_task.map(Message::MediaFocusDialogMessage),
                                ]);
                            }
                        }
                        image_cell::message::Message::ContextMenuOpen => {
                            match self.context_menu.state {
                                ContextMenuState::None => {
                                    if let Some(path) = &self.image_cell_path {
                                        self.context_menu.state =
                                            ContextMenuState::ImageCell(path.to_owned())
                                    }
                                }
                                _ => self.context_menu.state = ContextMenuState::None,
                            }
                        }
                    },
                    gallery::message::Message::CursorExit => self.image_cell_path_update(None),
                }

                task
            }
            Message::HeaderMessage(message) => {
                let task = self
                    .header
                    .update(message.clone())
                    .map(Message::HeaderMessage);

                match message {
                    header::message::Message::SimilarPairsDialogOpen => {
                        // todo: error handling
                        let dialog = similar_pairs_dialog::SimilarPairsDialog::new(
                            self.dir_node.clone().unwrap(),
                            None,
                        );
                        self.dialog = Some(Dialog::SimilarPairsDialog(dialog.clone()));
                        return dialog
                            .default_task()
                            .map(Message::SimilarPairsDialogMessage);
                    }
                    header::message::Message::SettingsOpen => {
                        self.dialog = Some(Dialog::SettingsDialog(
                            settings_dialog::SettingsDialog::new(
                                &self.settings.target_media_type,
                                self.settings.sub_dir_depth_limit,
                            ),
                        ))
                    }
                    _ => (),
                }

                task
            }
            Message::AsideMessage(message) => {
                let task = self
                    .aside
                    .update(message.clone())
                    .map(Message::AsideMessage);

                match message {
                    aside::message::Message::DirTreeMessage(message) => {
                        match message {
                            dir_tree::message::Message::DirClick(path) => {
                                self.settings.root_dir_path = path.to_string_lossy().to_string();
                                self.save_settings();

                                // todo dir_node should be got from dir_tree
                                let mut extension_allowlist: Vec<&str> = vec![];
                                if self.settings.target_media_type.include_image {
                                    extension_allowlist.extend(IMAGE_EXTENSION_ALLOWLIST);
                                }
                                if self.settings.target_media_type.include_video {
                                    extension_allowlist.extend(VIDEO_EXTENSION_ALLOWLIST);
                                }

                                let recurse = if 0 < self.settings.sub_dir_depth_limit {
                                    Recurse {
                                        enabled: true,
                                        depth_limit: Some(self.settings.sub_dir_depth_limit.into()),
                                    }
                                } else {
                                    Recurse {
                                        enabled: false,
                                        depth_limit: None,
                                    }
                                };

                                let dir_node = Swdir::default()
                                    .set_root_path(path)
                                    .set_extension_allowlist(&extension_allowlist)
                                    .expect("failed to set allowlist")
                                    .set_recurse(recurse)
                                    .walk();

                                let dir_node_count = dir_node.count();
                                self.footer
                                    .update_count(dir_node_count.files, dir_node_count.dirs);

                                self.dir_node = Some(dir_node);

                                return if self.processing {
                                    task
                                } else {
                                    self.processing_on();
                                    Task::batch([Task::done(Message::CacheRequire), task])
                                };
                            }
                            _ => (),
                        }
                    }
                }

                task
            }
            Message::FooterMessage(message) => {
                let task = self
                    .footer
                    .update(message.clone())
                    .map(|message| Message::FooterMessage(message));

                match message {
                    footer::message::Message::ThumbnailSizeChanged(value) => {
                        self.thumbnail_size_update(value)
                    }
                    _ => (),
                }

                task
            }
            Message::MediaFocusDialogMessage(message) => {
                if let Some(Dialog::MediaFocusDialog(x)) = &mut self.dialog {
                    let task = x
                        .update(message.clone())
                        .map(Message::MediaFocusDialogMessage);

                    match message {
                        media_focus_dialog::message::Message::CloseClick => self.dialog = None,
                        _ => (),
                    }

                    return task;
                }
                Task::none()
            }
            Message::SimilarPairsDialogMessage(message) => {
                if let Some(Dialog::SimilarPairsDialog(x)) = &mut self.dialog {
                    let task = x
                        .update(message.clone())
                        .map(Message::SimilarPairsDialogMessage);

                    match message {
                        similar_pairs_dialog::message::Message::MediaItemDoubleClicked(path) => {
                            let media_focus_dialog =
                                media_focus_dialog::MediaFocusDialog::new(path);
                            let dialog = Dialog::MediaFocusDialog(media_focus_dialog.clone());
                            let default_task = media_focus_dialog.default_task();

                            self.dialog = Some(dialog);

                            return Task::batch([
                                task,
                                default_task.map(Message::MediaFocusDialogMessage),
                            ]);
                        }
                        _ => (),
                    }

                    return task;
                }
                Task::none()
            }
            Message::SettingsDialogMessage(message) => {
                if let Some(Dialog::SettingsDialog(settings)) = &mut self.dialog {
                    let task = settings
                        .update(message.clone())
                        .map(Message::SettingsDialogMessage);

                    match message {
                        settings_dialog::message::Message::TargetMediaTypeChanged(x) => {
                            self.settings.target_media_type = x;
                            self.save_settings();

                            return if self.processing {
                                task
                            } else {
                                self.processing_on();
                                Task::batch([task, Task::done(Message::CacheRequire)])
                            };
                        }
                        settings_dialog::message::Message::SubDirDepthLimitChanged(x) => {
                            self.settings.sub_dir_depth_limit = x;
                            self.save_settings();

                            return if self.processing {
                                task
                            } else {
                                self.processing_on();
                                Task::batch([task, Task::done(Message::CacheRequire)])
                            };
                        }
                        _ => (),
                    }

                    return task;
                }
                Task::none()
            }
            Message::ContextMenuMessage(message) => self
                .context_menu
                .update(message)
                .map(Message::ContextMenuMessage),
            Message::DialogClose => {
                self.dialog = None;
                Task::none()
            }
            Message::CursorMove(point) => {
                match self.context_menu.state {
                    ContextMenuState::None => self.context_menu.update_point(point),
                    _ => (),
                };
                Task::none()
            }
        }
    }
}

fn dir_path_thumbnail_path_map(
    dir_node: &DirNode,
    image_cache_reader: &ImageCacheReader,
    video_cache_reader: &VideoCacheReader,
) -> BTreeMap<PathBuf, FastHashMap<String, String>> {
    let mut map = FastHashMap::default();

    for path in &dir_node.files {
        let thumbnail_path = if VIDEO_EXTENSION_ALLOWLIST.contains(
            &path
                .extension()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string()
                .as_str(),
        ) {
            match video_cache_reader.lookup(&path) {
                Ok(LookupResult::Hit(x)) if x.thumbnail_path.is_some() => {
                    PathBuf::from(x.thumbnail_path.unwrap())
                }
                _ => path.to_path_buf(),
            }
        } else {
            match image_cache_reader.lookup(&path) {
                Ok(LookupResult::Hit(x)) if x.thumbnail_path.is_some() => {
                    PathBuf::from(x.thumbnail_path.unwrap())
                }
                _ => path.to_path_buf(),
            }
        };

        map.insert(
            path.canonicalize()
                .expect("failed to canonicalize path")
                .to_string_lossy()
                .to_string(),
            thumbnail_path
                .canonicalize()
                .expect("failed to canonicalize thumbnail path")
                .to_string_lossy()
                .to_string(),
        );
    }

    let mut ret = BTreeMap::default();
    ret.insert(dir_node.path.to_owned(), map);

    for dir_node in &dir_node.sub_dirs {
        ret.extend(dir_path_thumbnail_path_map(
            dir_node,
            image_cache_reader,
            video_cache_reader,
        ));
    }

    ret
}
