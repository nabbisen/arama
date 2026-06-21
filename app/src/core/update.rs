use std::{collections::BTreeMap, path::PathBuf, sync::Arc};

use arama_ai::{
    model::model_container::clip, pipeline::encode::image::embeddings::image_embedding,
};
use arama_cache::{
    DbLocation, ImageCacheReader, ImageCacheWriter, LookupResult, UpsertImageRequest,
    VideoCacheReader,
};
use arama_env::{IMAGE_EXTENSION_ALLOWLIST, VIDEO_EXTENSION_ALLOWLIST, cache_storage_path};
use arama_ui_main::{
    components::gallery::image_cell,
    views::{cache_page, gallery},
};
use iced::{Task, wgpu::naga::FastHashMap};
use swdir::{DirNode, FilterRule, Recurse, Swdir};

use arama_i18n::set_locale;
use super::{App, Dialog, NavPage, message::Message};
use arama_ui_layout::{aside, footer, header};
use arama_ui_widgets::{
    context_menu::ContextMenuState,
    dialog::{media_focus_dialog, settings_dialog, similar_pairs_dialog},
};

impl App {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::NavTo(page) => {
                let reload = if page == NavPage::Cache {
                    self.cache_page.load_task().map(Message::CachePageMessage)
                } else {
                    Task::none()
                };
                self.nav_page = page;
                reload
            }
            Message::CacheRequire(target) => {
                let node = target.or_else(|| self.dir_node.clone());
                if let Some(dir_node) = node {

                    let (task, handle) = Task::perform(
                        async move {
                            let Ok(writer) =
                                ImageCacheWriter::onetime(arama_cache::DbLocation::Custom(
                                    cache_storage_path()
                                        .expect("failed to get cache storage path"),
                                ))
                            else {
                                return vec![];
                            };
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
                    .abortable();
                    self.task_handle = Some(handle);
                    task
                } else {
                    self.processing_off();
                    Task::none()
                }
            }
            Message::ThumbnailCacheFinished(ret) => {
                let errors: Vec<_> = ret.iter().filter(|x| x.1.is_err()).collect();
                if !errors.is_empty() {
                    let detail = errors
                        .iter()
                        .map(|x| format!("{:?}", x.1))
                        .collect::<Vec<_>>()
                        .join(", ");
                    self.push_error_toast("Cache error", detail);
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
                    let (task, handle) = Task::perform(
                        async {
                            image_embedding(ret.into_iter().map(|x| x.0).collect())
                                .await
                                .expect("failed to get embedding")
                        },
                        Message::EmbeddingCacheFinished,
                    )
                    .abortable();
                    self.task_handle = Some(handle);
                    task
                } else {
                    self.task_handle = None;
                    self.processing_off();
                    self.run_finished_reload()
                }
            }
            Message::EmbeddingCacheFinished(err) => {
                if let Some(err) = err {
                    self.push_error_toast("Embedding error", err);
                }

                self.task_handle = None;
                self.aside.set_processing(self.processing);
                self.header
                    .set_embedding_cached(self.gallery.embedding_cached());

                self.processing_off();
                self.run_finished_reload()
            }
            Message::CachePageMessage(message) => {
                let task = self
                    .cache_page
                    .update(message.clone())
                    .map(Message::CachePageMessage);

                match message {
                    cache_page::message::Message::Event(event) => match event {
                        cache_page::message::Event::CacheRequest(path) => {
                            return Task::batch([task, self.on_cache_page_request(path)]);
                        }
                        cache_page::message::Event::ClearRequest(dir) => {
                            return Task::batch([task, clear_dir_task(dir)]);
                        }
                        cache_page::message::Event::StopRequest => {
                            if let Some(handle) = self.task_handle.take() {
                                handle.abort();
                            }
                            let reload = self.run_finished_reload();
                            return Task::batch([task, reload]);
                        }
                    },
                    cache_page::message::Message::Internal(_) => (),
                }

                task
            }
            Message::CacheClearFinished(result) => {
                if let Err(err) = result {
                    self.push_error_toast("Cache clear failed", err);
                }
                // Reload so partial deletions are shown truthfully.
                self.cache_page.load_task().map(Message::CachePageMessage)
            }
            Message::SetupMessage(message) => {
                let task = self
                    .setup
                    .update(message.clone())
                    .map(Message::SetupMessage);
                if self.setup.finished {
                    self.processing_on();
                    Task::done(Message::CacheRequire(None))
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
                    gallery::message::Message::FilterChanged(_)
                    | gallery::message::Message::FilterClear => (),
                    gallery::message::Message::ImageCellMessage(message) => match message {
                        image_cell::message::Message::ImageCellEnter(path) => {
                            self.image_cell_path_update(Some(path));
                        }
                        image_cell::message::Message::ImageSelect => {
                            if let Some(path) = &self.image_cell_path {
                                let media_focus_dialog = media_focus_dialog::MediaFocusDialog::new(
                                    path,
                                    self.settings.cache_lookup_strategy,
                                    self.settings.similarity_threshold,
                                );
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
                    header::message::Message::Event(message) => {
                        match message {
                            header::message::Event::DirSelect(path) => {
                                self.aside.update_dir_tree(&path);
                                return self.on_dir_changed(path, task);
                            }
                            header::message::Event::SimilarPairsDialogOpen => {
                                let Some(dir_node) = self.dir_node.clone() else {
                                    self.push_error_toast(
                                        "Similarity pairs",
                                        "Select a directory first.".to_owned(),
                                    );
                                    return task;
                                };
                                let dialog = similar_pairs_dialog::SimilarPairsDialog::new(
                                    dir_node,
                                    None,
                                    self.settings.similarity_threshold,
                                );
                                self.dialog = Some(Dialog::SimilarPairsDialog(dialog.clone()));
                                return dialog
                                    .default_task()
                                    .map(Message::SimilarPairsDialogMessage);
                            }
                        }
                    }
                    header::message::Message::Internal(_) => (),
                }

                task
            }
            Message::AsideMessage(message) => {
                let task = self
                    .aside
                    .update(message.clone())
                    .map(Message::AsideMessage);

                match message {
                    aside::message::Message::Event(message) => match message {
                        aside::message::Event::DirSelect(path) => {
                            return self.on_dir_changed(path, task);
                        }
                    },
                    aside::message::Message::Internal(_) => (),
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
                        media_focus_dialog::message::Message::CacheLookupStrategyChanged(x) => {
                            self.settings.cache_lookup_strategy = x;
                            self.save_settings();
                        }
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
                            let media_focus_dialog = media_focus_dialog::MediaFocusDialog::new(
                                path,
                                self.settings.cache_lookup_strategy,
                                self.settings.similarity_threshold,
                            );
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
                let task = self
                    .settings_page
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
                            Task::batch([task, Task::done(Message::CacheRequire(None))])
                        };
                    }
                    settings_dialog::message::Message::SubDirDepthLimitChanged(x) => {
                        self.settings.sub_dir_depth_limit = x;
                        self.save_settings();

                        return if self.processing {
                            task
                        } else {
                            self.processing_on();
                            Task::batch([task, Task::done(Message::CacheRequire(None))])
                        };
                    }
                    settings_dialog::message::Message::SimilarityThresholdChanged(v) => {
                        self.settings.similarity_threshold = v;
                        self.save_settings();
                    }
                    settings_dialog::message::Message::LocaleChanged(l) => {
                        self.settings.locale = l;
                        set_locale(l);
                        self.save_settings();
                    }
                    settings_dialog::message::Message::ThemeChanged(theme) => {
                        self.settings.theme = theme;
                        arama_theme::set_theme(theme);
                        self.save_settings();
                    }
                    _ => (),
                }

                task
            }
            Message::ContextMenuMessage(message) => self
                .context_menu
                .update(message)
                .map(Message::ContextMenuMessage),
            Message::DialogClose => {
                self.dialog = None;
                Task::none()
            }
            Message::CloseMenus => {
                self.context_menu.state = ContextMenuState::None;
                Task::none()
            }
            Message::ToastDismiss(id) => {
                self.toasts.retain(|t| t.id != id);
                Task::none()
            }
            Message::ToastSweep => {
                snora::toast::sweep_expired(&mut self.toasts, std::time::Instant::now());
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

    fn on_dir_changed(&mut self, path: PathBuf, task: Task<Message>) -> Task<Message> {
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
            Recurse::Depth(self.settings.sub_dir_depth_limit as usize)
        } else {
            Recurse::None
        };

        let dir_node = Swdir::new()
            .root_path(path.clone())
            .filter(
                FilterRule::extension_allowlist(extension_allowlist.iter().copied())
                    .expect("failed to set allowlist"),
            )
            .recurse(recurse)
            .walk()
            .into_tree();

        let dir_node_count = dir_node.count();
        self.footer
            .update_count(dir_node_count.files, dir_node_count.dirs);

        self.dir_node = Some(dir_node);

        // Reset the gallery search filter for the new directory.
        self.gallery.clear_filter();

        // Abort any running indexing task: the user switched directories,
        // so the old result is no longer wanted.
        if let Some(handle) = self.task_handle.take() {
            handle.abort();
        }
        // Mark the run on the Cache page (RFC 004 ⏳ indicator).
        self.cache_page.run_started(path);
        self.processing_on();
        Task::batch([Task::done(Message::CacheRequire(None)), task])
    }

    /// Handle a Cache-page request to index `path`: validate, mark the
    /// run, abort any in-flight task, and start the pipeline with an
    /// explicit target — without touching the Explorer's selection.
    fn on_cache_page_request(&mut self, path: PathBuf) -> Task<Message> {
        if !path.is_dir() {
            self.push_error_toast(
                "Invalid directory",
                format!("Not an existing directory: {}", path.display()),
            );
            return Task::none();
        }

        let mut extension_allowlist: Vec<&str> = vec![];
        if self.settings.target_media_type.include_image {
            extension_allowlist.extend(IMAGE_EXTENSION_ALLOWLIST);
        }
        if self.settings.target_media_type.include_video {
            extension_allowlist.extend(VIDEO_EXTENSION_ALLOWLIST);
        }
        let recurse = if 0 < self.settings.sub_dir_depth_limit {
            Recurse::Depth(self.settings.sub_dir_depth_limit as usize)
        } else {
            Recurse::None
        };
        let node = Swdir::new()
            .root_path(path.clone())
            .filter(
                FilterRule::extension_allowlist(extension_allowlist.iter().copied())
                    .expect("failed to set allowlist"),
            )
            .recurse(recurse)
            .walk()
            .into_tree();

        // Single-task rule: a new run replaces any in-flight one.
        if let Some(handle) = self.task_handle.take() {
            handle.abort();
        }
        self.cache_page.run_started(path);
        self.processing_on();
        Task::done(Message::CacheRequire(Some(node)))
    }

    /// At the end of an indexing run: clear the ⏳ marker and reload
    /// the Cache page table when it has been visited at least once.
    fn run_finished_reload(&mut self) -> Task<Message> {
        self.cache_page.run_finished();
        if self.cache_page.is_loaded() {
            self.cache_page.load_task().map(Message::CachePageMessage)
        } else {
            Task::none()
        }
    }
}

/// Async per-directory clear across both cache namespaces.
fn clear_dir_task(dir: PathBuf) -> Task<Message> {
    Task::perform(
        async move {
            let location = arama_cache::DbLocation::Custom(
                cache_storage_path().map_err(|e| e.to_string())?,
            );
            let removed_images = ImageCacheWriter::onetime(location.clone())
                .map_err(|e| e.to_string())?
                .delete_in_dir(&dir)
                .map_err(|e| e.to_string())?;
            let removed_videos =
                arama_cache::VideoCacheWriter::onetime(location, None, None)
                    .map_err(|e| e.to_string())?
                    .delete_in_dir(&dir)
                    .map_err(|e| e.to_string())?;
            Ok(removed_images + removed_videos)
        },
        Message::CacheClearFinished,
    )
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
            thumbnail_path.to_string_lossy().to_string(),
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
