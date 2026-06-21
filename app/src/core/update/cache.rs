use std::{collections::BTreeMap, path::PathBuf, sync::Arc};

use arama_ai::{
    model::model_container::clip, pipeline::encode::image::embeddings::image_embedding,
};
use arama_cache::{
    DbLocation, ImageCacheReader, ImageCacheWriter, LookupResult, UpsertImageRequest,
    VideoCacheReader,
};
use arama_env::{IMAGE_EXTENSION_ALLOWLIST, VIDEO_EXTENSION_ALLOWLIST, cache_storage_path};
use arama_ui_main::views::cache_page;
use iced::{Task, wgpu::naga::FastHashMap};
use swdir::{DirNode, FilterRule, Recurse, Swdir};

use super::super::{App, message::Message};

impl App {
    pub(super) fn handle_cache_require(&mut self, target: Option<DirNode>) -> Task<Message> {
        let node = target.or_else(|| self.dir_node.clone());
        if let Some(dir_node) = node {
            let (task, handle) = Task::perform(
                async move {
                    let Ok(writer) = ImageCacheWriter::onetime(arama_cache::DbLocation::Custom(
                        cache_storage_path().expect("failed to get cache storage path"),
                    )) else {
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

    pub(super) fn handle_thumbnail_cache_finished(
        &mut self,
        ret: Vec<(PathBuf, Arc<arama_cache::Result<()>>)>,
    ) -> Task<Message> {
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
                    dir_node,
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

    pub(super) fn handle_embedding_cache_finished(&mut self, err: Option<String>) -> Task<Message> {
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

    pub(super) fn handle_cache_page_message(
        &mut self,
        message: cache_page::message::Message,
    ) -> Task<Message> {
        let task = self
            .cache_page
            .update(message.clone())
            .map(Message::CachePageMessage);

        match message {
            cache_page::message::Message::Event(event) => match event {
                cache_page::message::Event::CacheRequest(path) => {
                    Task::batch([task, self.on_cache_page_request(path)])
                }
                cache_page::message::Event::ClearRequest(dir) => {
                    Task::batch([task, clear_dir_task(dir)])
                }
                cache_page::message::Event::StopRequest => {
                    if let Some(handle) = self.task_handle.take() {
                        handle.abort();
                    }
                    let reload = self.run_finished_reload();
                    Task::batch([task, reload])
                }
            },
            cache_page::message::Message::Internal(_) => task,
        }
    }

    pub(super) fn handle_cache_clear_finished(
        &mut self,
        result: Result<usize, String>,
    ) -> Task<Message> {
        if let Err(err) = result {
            self.push_error_toast("Cache clear failed", err);
        }
        // Reload so partial deletions are shown truthfully.
        self.cache_page.load_task().map(Message::CachePageMessage)
    }

    /// Switch to a new directory from the Explorer: update settings, rebuild
    /// the dir-node, reset gallery filter, abort any in-flight task, and start
    /// the cache pipeline.
    pub(super) fn on_dir_changed(&mut self, path: PathBuf, task: Task<Message>) -> Task<Message> {
        self.settings.root_dir_path = path.to_string_lossy().to_string();
        self.save_settings();

        // todo dir_node should be got from dir_tree
        let dir_node = build_dir_node(&path, &self.settings);

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

        let node = build_dir_node(&path, &self.settings);

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
    pub(super) fn run_finished_reload(&mut self) -> Task<Message> {
        self.cache_page.run_finished();
        if self.cache_page.is_loaded() {
            self.cache_page.load_task().map(Message::CachePageMessage)
        } else {
            Task::none()
        }
    }
}

/// Build a `DirNode` for `path` using the current media-type and depth
/// settings. Extracted to remove duplication between `on_dir_changed` and
/// `on_cache_page_request`.
fn build_dir_node(path: &PathBuf, settings: &arama_env::Settings) -> DirNode {
    let mut extension_allowlist: Vec<&str> = vec![];
    if settings.target_media_type.include_image {
        extension_allowlist.extend(IMAGE_EXTENSION_ALLOWLIST);
    }
    if settings.target_media_type.include_video {
        extension_allowlist.extend(VIDEO_EXTENSION_ALLOWLIST);
    }

    let recurse = if 0 < settings.sub_dir_depth_limit {
        Recurse::Depth(settings.sub_dir_depth_limit as usize)
    } else {
        Recurse::None
    };

    Swdir::new()
        .root_path(path.clone())
        .filter(
            FilterRule::extension_allowlist(extension_allowlist.iter().copied())
                .expect("failed to set allowlist"),
        )
        .recurse(recurse)
        .walk()
        .into_tree()
}

/// Async per-directory clear across both cache namespaces.
pub(super) fn clear_dir_task(dir: PathBuf) -> Task<Message> {
    Task::perform(
        async move {
            let location =
                arama_cache::DbLocation::Custom(cache_storage_path().map_err(|e| e.to_string())?);
            let removed_images = ImageCacheWriter::onetime(location.clone())
                .map_err(|e| e.to_string())?
                .delete_in_dir(&dir)
                .map_err(|e| e.to_string())?;
            let removed_videos = arama_cache::VideoCacheWriter::onetime(location, None, None)
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
            match video_cache_reader.lookup(path) {
                Ok(LookupResult::Hit(x)) if x.thumbnail_path.is_some() => {
                    PathBuf::from(x.thumbnail_path.unwrap())
                }
                _ => path.to_path_buf(),
            }
        } else {
            match image_cache_reader.lookup(path) {
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
