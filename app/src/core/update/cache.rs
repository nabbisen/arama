use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use arama_ai::{
    model::model_container::clip,
    pipeline::encode::image::embeddings::{EmbeddingRunReport, image_embedding},
};
use arama_cache::{
    CacheMaintenance, CachePruneRequest, DbLocation, ImageCacheReader, ImageCacheWriter,
    LookupResult, UpsertImageRequest, VideoCacheReader,
};
use arama_env::{
    IMAGE_EXTENSION_ALLOWLIST, VIDEO_EXTENSION_ALLOWLIST, cache_dir, cache_storage_path,
    cache_thumbnail_dir_path, diagnostic,
};
use arama_i18n::{t, t_with};
use arama_ui_main::views::cache_page;
use iced::{Task, wgpu::naga::FastHashMap};
use swdir::{DirNode, FilterRule, Recurse, Swdir};

use super::super::{App, message::Message};

/// RFC 047 S3.2: how many `UpsertImageRequest`s one `spawn_blocking` call
/// processes before the loop in `handle_cache_require` checks whether it
/// should stop. Each chunk's own rayon fan-out (inside `upsert_all`) is
/// not itself interruptible once dispatched - this is the bound on how
/// much work can still be "in flight and unstoppable" at the moment Stop
/// is pressed. Picked empirically (T061 review package): small enough to
/// keep that bound well under a second on real files, large enough that
/// per-chunk dispatch and SQLite-write overhead stay a small fraction of
/// total run time on a steady-state (already-cached) re-index.
const THUMBNAIL_CHUNK_SIZE: usize = 24;

/// RFC 047 S3.1/S3.2: the thumbnail phase, extracted to a free function
/// (rather than inlined in `handle_cache_require`'s `Task::perform`) so
/// it can be driven directly by a manual `tokio` runtime in tests -
/// `iced::Task` has no public API to poll one to completion outside the
/// real application executor, but a plain `async fn` needs none.
///
/// Chunked so there is an `.await` point between groups of files, which
/// is what lets dropping the `Task` built around this (Stop, or a
/// directory switch) actually stop the *next* chunk from ever being
/// dispatched - the previous whole-batch `upsert_all` call had no await
/// point anywhere inside it, so `Task::abortable()`'s drop had nothing to
/// act on and the phase ran to completion regardless of Stop.
///
/// `cancel_flag` is checked explicitly rather than relying only on that
/// drop, because it is the authoritative, directly-testable signal: a
/// chunk already dispatched to `spawn_blocking` keeps running on its own
/// thread even after this future stops being polled (that is what "rayon
/// work already dispatched... continues" means in practice), so checking
/// the flag *before* the next chunk's `spawn_blocking` call is what
/// actually bounds how much further work can start.
async fn run_thumbnail_phase(
    dir_node: DirNode,
    cancel_flag: Arc<AtomicBool>,
) -> Vec<(PathBuf, Arc<arama_cache::Result<()>>)> {
    let Ok(cache_path) = cache_storage_path() else {
        return vec![];
    };
    let Ok(writer) = ImageCacheWriter::onetime(arama_cache::DbLocation::Custom(cache_path)) else {
        return vec![];
    };
    let paths = dir_node.flatten_paths();

    let mut results: Vec<(PathBuf, Arc<arama_cache::Result<()>>)> = Vec::new();
    for path_chunk in paths.chunks(THUMBNAIL_CHUNK_SIZE) {
        if cancel_flag.load(Ordering::Relaxed) {
            break;
        }
        let chunk: Vec<UpsertImageRequest> = path_chunk
            .iter()
            .map(|path| UpsertImageRequest {
                path: path.to_path_buf(),
                clip_vector: None,
            })
            .collect();
        let writer = writer.clone();
        let Ok(chunk_results) = tokio::task::spawn_blocking(move || writer.upsert_all(chunk)).await
        else {
            // The blocking task panicked. Keep whatever earlier chunks
            // already produced rather than losing it too, and stop - a
            // panic means something is genuinely wrong, not a reason to
            // keep dispatching more chunks against it.
            break;
        };
        results.extend(chunk_results.into_iter().map(|(p, r)| (p, Arc::new(r))));
    }
    results
}

impl App {
    pub(super) fn handle_cache_require(&mut self, target: Option<DirNode>) -> Task<Message> {
        let node = target.or_else(|| self.dir_node.clone());
        if let Some(dir_node) = node {
            let cancel_flag = Arc::new(AtomicBool::new(false));
            self.cancel_flag = Some(cancel_flag.clone());
            let (task, handle) = Task::perform(
                run_thumbnail_phase(dir_node, cancel_flag),
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
            self.push_error_toast(t("toast.cache_error.title"), detail);
        }

        if let Some(dir_node) = &self.dir_node {
            match cache_storage_path() {
                Ok(cache_path) => {
                    let image_cache_reader =
                        ImageCacheReader::onetime(DbLocation::Custom(cache_path.clone()));
                    let video_cache_reader =
                        VideoCacheReader::onetime(DbLocation::Custom(cache_path));

                    match (image_cache_reader, video_cache_reader) {
                        (Ok(image_cache_reader), Ok(video_cache_reader)) => {
                            self.gallery.set_dir_path_thumbnail_path_map(
                                dir_path_thumbnail_path_map(
                                    dir_node,
                                    &image_cache_reader,
                                    &video_cache_reader,
                                ),
                            );

                            // RFC 047 S1.4/S3.4: this used to pass
                            // `self.gallery.embedding_cached()` straight
                            // through here - a file-count check with no
                            // relation to whether embedding ever ran -
                            // which is exactly why the button used to
                            // ungrey ~35 lines before the embedding task
                            // below is even spawned. Explicit `false`:
                            // embedding has not started yet at this
                            // point, let alone completed. The only place
                            // that may set this `true` is
                            // `handle_embedding_cache_finished`, once the
                            // embedding phase has actually completed.
                            self.header.set_embedding_cached(false);
                        }
                        (Err(err), _) => {
                            self.push_error_toast(
                                t("toast.cache_reload_failed.title"),
                                format!(
                                    "{}: {err}",
                                    t("toast.cache_reload_failed.image_reader.body")
                                ),
                            );
                        }
                        (_, Err(err)) => {
                            self.push_error_toast(
                                t("toast.cache_reload_failed.title"),
                                format!(
                                    "{}: {err}",
                                    t("toast.cache_reload_failed.video_reader.body")
                                ),
                            );
                        }
                    }
                }
                Err(err) => {
                    self.push_error_toast(
                        t("toast.cache_reload_failed.title"),
                        format!(
                            "{}: {err}",
                            t("toast.cache_reload_failed.storage_path.body")
                        ),
                    );
                }
            }
        }

        if clip::model().ready().unwrap_or(false) && !ret.is_empty() {
            let ffmpeg_toolchain = self.ffmpeg_authority.toolchain().cloned();
            let (task, handle) = Task::perform(
                async move {
                    image_embedding(ret.into_iter().map(|x| x.0).collect(), ffmpeg_toolchain)
                        .await
                        .map_err(|err| format!("{}: {err}", t("toast.embedding_error.body")))
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

    pub(super) fn handle_embedding_cache_finished(
        &mut self,
        result: Result<EmbeddingRunReport, String>,
    ) -> Task<Message> {
        match result {
            Ok(report) if report.has_warnings() => {
                self.push_warning_toast(
                    t("toast.indexed_with_warnings.title"),
                    embedding_report_summary(&report),
                );
            }
            Ok(_) => (),
            Err(err) => {
                self.push_error_toast(t("toast.embedding_error.title"), err);
            }
        }

        self.task_handle = None;
        self.cancel_flag = None;
        // Reaching this handler at all means the embedding `Task` was
        // never aborted (RFC 047 S7.1): `Handle::abort` drops the future
        // without ever delivering its completion message, confirmed
        // against `iced_runtime`'s own `Abortable`-based implementation.
        // So "this handler ran" already *is* "embedding completed, not
        // merely started" - no separate cancelled/not-cancelled check is
        // needed here.
        self.embedding_completed = true;
        self.aside.set_processing(self.processing);
        self.header
            .set_embedding_cached(self.embedding_completed && self.gallery.embedding_cached());

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
                cache_page::message::Event::PruneRequest(max_bytes) => {
                    Task::batch([task, prune_task(max_bytes)])
                }
                cache_page::message::Event::StopRequest => {
                    self.abort_indexing();
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
            self.push_error_toast(t("toast.cache_clear_failed.title"), err);
        }
        // Reload so partial deletions are shown truthfully.
        self.cache_page.load_task().map(Message::CachePageMessage)
    }

    pub(super) fn handle_cache_prune_finished(
        &mut self,
        result: Result<arama_cache::CachePruneReport, String>,
    ) -> Task<Message> {
        match result {
            Ok(report) => {
                self.cache_page.prune_finished(Some(report));
                if report.target_reached {
                    self.push_success_toast(
                        t("toast.cache_prune_complete.title"),
                        cache_prune_complete_body(
                            report.removed_entries,
                            &human_size(report.after.total_bytes),
                        ),
                    );
                } else {
                    self.push_warning_toast(
                        t("toast.cache_prune_partial.title"),
                        cache_prune_partial_body(
                            report.removed_entries,
                            &human_size(report.unreclaimable_bytes),
                        ),
                    );
                }
            }
            Err(err) => {
                self.cache_page.prune_finished(None);
                self.push_error_toast(t("toast.cache_prune_failed.title"), err);
            }
        }
        self.cache_page.load_task().map(Message::CachePageMessage)
    }

    /// Task 039: the confirm dialog is already closed by the time this
    /// runs (`handle_confirm_dialog_message`) - this only reports the
    /// outcome. Unlike clear/prune, nothing here reloads the Cache page
    /// or the Explorer gallery: this task's own scope is the Settings
    /// page, whose "Delete cache" button already re-evaluates whether the
    /// cache directory exists on every render, with no stored state to
    /// go stale.
    pub(super) fn handle_cache_delete_finished(
        &mut self,
        result: Result<(), String>,
    ) -> Task<Message> {
        match result {
            Ok(()) => self.push_success_toast(
                t("toast.cache_delete_complete.title"),
                t("toast.cache_delete_complete.body"),
            ),
            Err(err) => self.push_error_toast(t("toast.cache_delete_failed.title"), err),
        }
        Task::none()
    }

    /// Switch to a new directory from the Explorer: update settings, rebuild
    /// the dir-node, reset gallery filter, abort any in-flight task, and start
    /// the cache pipeline.
    pub(super) fn on_dir_changed(&mut self, path: PathBuf, task: Task<Message>) -> Task<Message> {
        self.settings.root_dir_path = path.to_string_lossy().to_string();
        self.save_settings();

        // Keep the header path input in sync regardless of how navigation
        // was triggered (header submit, file-picker, or aside tree click).
        self.header.set_path(&path.to_string_lossy());

        // The aside tree is folder-only UI state; cache/gallery indexing needs
        // a separate media-extension-aware scan.
        let dir_node = build_dir_node(&path, &self.settings);

        let dir_node_count = dir_node.count();
        self.footer
            .update_count(dir_node_count.files, dir_node_count.dirs);

        self.dir_node = Some(dir_node);

        // Reset the gallery search filter for the new directory.
        self.gallery.clear_filter();

        // Abort any running indexing task: the user switched directories,
        // so the old result is no longer wanted (RFC 047 S1: both the
        // thumbnail and embedding phases must stop, not only embedding).
        self.abort_indexing();
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
                t("toast.invalid_directory.title"),
                format!("{}: {}", t("toast.invalid_directory.body"), path.display()),
            );
            return Task::none();
        }

        let node = build_dir_node(&path, &self.settings);

        // Single-task rule: a new run replaces any in-flight one.
        self.abort_indexing();
        self.cache_page.run_started(path);
        self.processing_on();
        Task::done(Message::CacheRequire(Some(node)))
    }

    /// RFC 047 S1/S3.2/S3.4: stop whichever indexing phase (thumbnail or
    /// embedding) is currently in flight, shared by Stop and by a
    /// directory switch. Aborts the `Task` (stops the next unit of work
    /// it would start) *and* signals `cancel_flag` (stops the thumbnail
    /// loop's own next chunk dispatch - see that loop's comment for why
    /// both are needed). Also clears `embedding_completed` immediately,
    /// rather than waiting for a cache-finished handler that a Stop may
    /// prevent from ever running: the button must grey out the moment a
    /// run is known to be incomplete, not some indeterminate time later.
    fn abort_indexing(&mut self) {
        if let Some(handle) = self.task_handle.take() {
            handle.abort();
        }
        if let Some(flag) = self.cancel_flag.take() {
            flag.store(true, Ordering::Relaxed);
        }
        self.embedding_completed = false;
        self.header.set_embedding_cached(false);
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
fn build_dir_node(path: &Path, settings: &arama_env::Settings) -> DirNode {
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

    let scanner = Swdir::new().root_path(path.to_path_buf()).recurse(recurse);
    let scanner = match FilterRule::extension_allowlist(extension_allowlist.iter().copied()) {
        Ok(filter) => scanner.filter(filter),
        Err(err) => {
            diagnostic(&format!("failed to set extension allowlist: {err}"));
            scanner
        }
    };

    scanner.walk().into_tree()
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

pub(super) fn prune_task(max_bytes: u64) -> Task<Message> {
    Task::perform(
        async move {
            let location =
                arama_cache::DbLocation::Custom(cache_storage_path().map_err(|e| e.to_string())?);
            let thumbnail_dir = cache_thumbnail_dir_path().map_err(|e| e.to_string())?;
            CacheMaintenance::onetime(location, Some(thumbnail_dir))
                .map_err(|e| e.to_string())?
                .prune(CachePruneRequest { max_bytes })
                .map_err(|e| e.to_string())
        },
        Message::CachePruneFinished,
    )
}

/// Task 039: deletes the cache directory's *contents*, not the directory
/// itself - the directory must still exist afterwards (audit finding 4),
/// and re-indexing recreates whatever it needs inside it. Every entry is
/// attempted even if an earlier one fails, so one locked file (Windows,
/// most likely) does not leave the rest of a stale cache behind; the
/// first failure is what reaches the user as the outcome toast, and
/// every failure is also recorded via `arama_env::diagnostic` so a
/// Windows release build (Task 037) does not lose the detail silently.
pub(super) fn delete_cache_task() -> Task<Message> {
    Task::perform(
        async move {
            let path = cache_dir().map_err(|e| e.to_string())?;
            delete_dir_contents(&path)
        },
        Message::CacheDeleteFinished,
    )
}

/// The pure half of [`delete_cache_task`] - same seam this project uses
/// elsewhere (e.g. `env/src/dir.rs`'s `local_dir_with_override`): the
/// real caller resolves `cache_dir()` and calls this with the result,
/// tests supply an arbitrary temp directory directly.
fn delete_dir_contents(path: &Path) -> Result<(), String> {
    let entries = std::fs::read_dir(path).map_err(|e| e.to_string())?;
    let mut first_err: Option<String> = None;
    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(err) => {
                diagnostic(&format!("failed to read cache directory entry: {err}"));
                first_err.get_or_insert_with(|| err.to_string());
                continue;
            }
        };
        let entry_path = entry.path();
        let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
        let result = if is_dir {
            std::fs::remove_dir_all(&entry_path)
        } else {
            std::fs::remove_file(&entry_path)
        };
        if let Err(err) = result {
            diagnostic(&format!(
                "failed to remove cache entry {}: {err}",
                entry_path.display()
            ));
            first_err.get_or_insert_with(|| err.to_string());
        }
    }
    first_err.map_or(Ok(()), Err)
}

fn human_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while 1024.0 <= value && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{} {}", bytes, UNITS[unit])
    } else {
        format!("{:.1} {}", value, UNITS[unit])
    }
}

fn cache_prune_complete_body(removed_entries: usize, after_size: &str) -> String {
    t_with(
        "toast.cache_prune_complete.body",
        &[
            ("{count}", &removed_entries.to_string()),
            ("{size}", after_size),
        ],
    )
}

fn cache_prune_partial_body(removed_entries: usize, unreclaimable_size: &str) -> String {
    t_with(
        "toast.cache_prune_partial.body",
        &[
            ("{count}", &removed_entries.to_string()),
            ("{size}", unreclaimable_size),
        ],
    )
}

fn embedding_report_summary(report: &EmbeddingRunReport) -> String {
    let mut parts = Vec::new();
    if !report.skipped.is_empty() {
        parts.push(format!(
            "{} {}",
            report.skipped.len(),
            t("cache.summary_report.files_skipped")
        ));
    }
    if !report.cache_write_failures.is_empty() {
        parts.push(format!(
            "{} {}",
            report.cache_write_failures.len(),
            t("cache.summary_report.cache_writes_failed")
        ));
    }
    if parts.is_empty() {
        format!(
            "{} {}",
            report.processed,
            t("cache.summary_report.files_indexed")
        )
    } else {
        parts.join(", ")
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
            match video_cache_reader.lookup(path) {
                Ok(LookupResult::Hit(x)) => x
                    .thumbnail_path
                    .map(PathBuf::from)
                    .unwrap_or_else(|| path.to_path_buf()),
                _ => path.to_path_buf(),
            }
        } else {
            match image_cache_reader.lookup(path) {
                Ok(LookupResult::Hit(x)) => x
                    .thumbnail_path
                    .map(PathBuf::from)
                    .unwrap_or_else(|| path.to_path_buf()),
                _ => path.to_path_buf(),
            }
        };

        let key = path
            .canonicalize()
            .unwrap_or_else(|_| path.to_path_buf())
            .to_string_lossy()
            .to_string();
        map.insert(key, thumbnail_path.to_string_lossy().to_string());
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

#[cfg(test)]
mod tests {
    use arama_ai::pipeline::encode::image::embeddings::EmbeddingFileIssue;

    use super::{
        EmbeddingRunReport, cache_prune_complete_body, cache_prune_partial_body,
        delete_dir_contents, embedding_report_summary,
    };

    /// Task 039 acceptance criterion 4: the directory itself must survive.
    #[test]
    fn delete_dir_contents_removes_entries_but_keeps_the_directory_itself() {
        let dir = std::env::temp_dir().join(format!(
            "arama-cache-delete-test-{}-contents",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("thumbnail")).unwrap();
        std::fs::write(dir.join("thumbnail/a.jpg"), b"x").unwrap();
        std::fs::write(dir.join("cache-v2.sqlite"), b"x").unwrap();

        let result = delete_dir_contents(&dir);

        assert_eq!(result, Ok(()));
        assert!(dir.exists(), "the directory itself must still exist");
        assert_eq!(
            std::fs::read_dir(&dir).unwrap().count(),
            0,
            "every entry inside it must be gone"
        );
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn delete_dir_contents_on_an_already_empty_directory_succeeds() {
        let dir = std::env::temp_dir().join(format!(
            "arama-cache-delete-test-{}-empty",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        assert_eq!(delete_dir_contents(&dir), Ok(()));
        assert!(dir.exists());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    /// Task 039: "one locked file does not leave the rest of a stale
    /// cache behind" - a real permission failure on one entry must not
    /// stop the others from being removed. Unix-only: a 000-mode
    /// subdirectory is not a portable way to force this on every
    /// platform this project ships for, and this exact path was already
    /// verified live, once, in the review package (a real "Permission
    /// denied (os error 13)" toast).
    #[cfg(unix)]
    #[test]
    fn delete_dir_contents_removes_every_entry_it_can_even_when_one_entry_fails() {
        use std::os::unix::fs::PermissionsExt;

        let dir = std::env::temp_dir().join(format!(
            "arama-cache-delete-test-{}-partial-failure",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("removable.txt"), b"x").unwrap();
        let locked = dir.join("locked");
        std::fs::create_dir_all(locked.join("inner")).unwrap();
        std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o000)).unwrap();

        let result = delete_dir_contents(&dir);

        assert!(result.is_err(), "the locked entry's failure must surface");
        assert!(
            !dir.join("removable.txt").exists(),
            "the removable entry must not be left behind by the locked one's failure"
        );
        assert!(
            locked.exists(),
            "the locked entry itself is the one that could not be removed"
        );

        std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o755)).unwrap();
        std::fs::remove_dir_all(&dir).unwrap();
    }

    /// English-only, deliberately: this crate's own binary shares the
    /// `app` crate's much larger test suite, most of which asserts exact
    /// English text without expecting `arama_i18n`'s global locale to
    /// move under it. An earlier version of this test looped over both
    /// locales and was found to race those tests when the full workspace
    /// suite ran repeatedly. `toast.cache_prune_complete.body` and
    /// `toast.cache_prune_partial.body` - the two keys these functions
    /// call `t_with` on - are verified in both locales in
    /// `arama-i18n`'s own, much smaller test binary instead
    /// (`crates/i18n/src/lib.rs`'s `task_034_*` tests), where mutating
    /// the global locale is safe.
    #[test]
    fn cache_prune_bodies_substitute_correctly_in_english() {
        let complete = cache_prune_complete_body(42, "1.2 GB");
        assert!(complete.contains("42"), "{complete}");
        assert!(complete.contains("1.2 GB"), "{complete}");
        assert!(!complete.contains('{'), "{complete}");

        let partial = cache_prune_partial_body(7, "300.0 MB");
        assert!(partial.contains('7'), "{partial}");
        assert!(partial.contains("300.0 MB"), "{partial}");
        assert!(!partial.contains('{'), "{partial}");
    }

    #[test]
    fn embedding_report_summary_reads_correctly_in_english() {
        let skipped_only = EmbeddingRunReport {
            processed: 0,
            skipped: vec![EmbeddingFileIssue {
                path: "a.jpg".into(),
                message: "unreadable".to_owned(),
            }],
            cache_write_failures: vec![],
        };
        let summary = embedding_report_summary(&skipped_only);
        assert!(summary.starts_with('1'), "{summary}");
        assert!(!summary.contains('{'), "{summary}");

        let indexed_only = EmbeddingRunReport {
            processed: 5,
            skipped: vec![],
            cache_write_failures: vec![],
        };
        let summary = embedding_report_summary(&indexed_only);
        assert!(summary.starts_with('5'), "{summary}");
        assert!(!summary.contains('{'), "{summary}");
    }

    // --- RFC 047 S3.1/S3.2: `run_thumbnail_phase` cancellation ---------
    //
    // Drives the extracted async fn directly with a manual `tokio`
    // runtime - `iced::Task` has no public API to poll one to completion
    // outside the real application executor (confirmed: `iced::task`
    // only re-exports `Handle`/`Task`/`Never`/`Sipper`/`Straw`/`sipper`/
    // `stream`, not `into_stream`), so this is the level these
    // properties are actually checkable at. State assertions on the
    // function's own return value, not timing read from rendered frames,
    // per the GUI evidence rules' "assert, do not photograph in a loop"
    // rule.

    use std::path::PathBuf;
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };
    use std::time::{Duration, Instant};

    use arama_env::DATA_HOME_ENV_VAR;
    use swdir::DirNode;

    use super::super::super::ARAMA_DATA_HOME_TEST_LOCK;
    use super::{THUMBNAIL_CHUNK_SIZE, run_thumbnail_phase};

    /// A syntactically valid 1x1 JPEG - enough for `image::open` to
    /// decode and `generate_image_thumbnail` to resize successfully.
    /// Identical in spirit to `crates/cache/tests/helpers.rs`'s
    /// `MINIMAL_JPEG` (private to that crate, not reusable from here).
    const MINIMAL_JPEG: &[u8] = &[
        0xff, 0xd8, 0xff, 0xe0, 0x00, 0x10, 0x4a, 0x46, 0x49, 0x46, 0x00, 0x01, 0x01, 0x00, 0x00,
        0x01, 0x00, 0x01, 0x00, 0x00, 0xff, 0xdb, 0x00, 0x43, 0x00, 0x08, 0x06, 0x06, 0x07, 0x06,
        0x05, 0x08, 0x07, 0x07, 0x07, 0x09, 0x09, 0x08, 0x0a, 0x0c, 0x14, 0x0d, 0x0c, 0x0b, 0x0b,
        0x0c, 0x19, 0x12, 0x13, 0x0f, 0x14, 0x1d, 0x1a, 0x1f, 0x1e, 0x1d, 0x1a, 0x1c, 0x1c, 0x20,
        0x24, 0x2e, 0x27, 0x20, 0x22, 0x2c, 0x23, 0x1c, 0x1c, 0x28, 0x37, 0x29, 0x2c, 0x30, 0x31,
        0x34, 0x34, 0x34, 0x1f, 0x27, 0x39, 0x3d, 0x38, 0x32, 0x3c, 0x2e, 0x33, 0x34, 0x32, 0xff,
        0xc0, 0x00, 0x0b, 0x08, 0x00, 0x01, 0x00, 0x01, 0x01, 0x01, 0x11, 0x00, 0xff, 0xc4, 0x00,
        0x1f, 0x00, 0x00, 0x01, 0x05, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b,
        0xff, 0xc4, 0x00, 0x1f, 0x10, 0x00, 0x02, 0x01, 0x03, 0x03, 0x02, 0x04, 0x03, 0x05, 0x05,
        0x04, 0x04, 0x00, 0x00, 0x01, 0x7d, 0x01, 0x02, 0x03, 0x00, 0x04, 0x11, 0x05, 0x12, 0x21,
        0x31, 0x41, 0x06, 0x13, 0x51, 0x61, 0x07, 0x22, 0x71, 0x14, 0x32, 0x81, 0x91, 0xa1, 0x08,
        0x23, 0x42, 0xb1, 0xc1, 0x15, 0x52, 0xd1, 0xf0, 0x24, 0x33, 0x62, 0x72, 0x82, 0x09, 0x0a,
        0x16, 0x17, 0x18, 0x19, 0x1a, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2a, 0x34, 0x35, 0x36, 0x37,
        0x38, 0x39, 0x3a, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x4a, 0x53, 0x54, 0x55, 0x56,
        0x57, 0x58, 0x59, 0x5a, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6a, 0x73, 0x74, 0x75,
        0x76, 0x77, 0x78, 0x79, 0x7a, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88, 0x89, 0x8a, 0x92, 0x93,
        0x94, 0x95, 0x96, 0x97, 0x98, 0x99, 0x9a, 0xa2, 0xa3, 0xa4, 0xa5, 0xa6, 0xa7, 0xa8, 0xa9,
        0xaa, 0xb2, 0xb3, 0xb4, 0xb5, 0xb6, 0xb7, 0xb8, 0xb9, 0xba, 0xc2, 0xc3, 0xc4, 0xc5, 0xc6,
        0xc7, 0xc8, 0xc9, 0xca, 0xd2, 0xd3, 0xd4, 0xd5, 0xd6, 0xd7, 0xd8, 0xd9, 0xda, 0xe1, 0xe2,
        0xe3, 0xe4, 0xe5, 0xe6, 0xe7, 0xe8, 0xe9, 0xea, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7,
        0xf8, 0xf9, 0xfa, 0xff, 0xda, 0x00, 0x08, 0x01, 0x01, 0x00, 0x00, 0x3f, 0x00, 0xfb, 0xd0,
        0x3f, 0xff, 0xd9,
    ];

    /// Isolated `ARAMA_DATA_HOME` plus `n` real, decodable image files on
    /// disk, under their own scratch source directory (never the cache
    /// scratch dir itself). Returns the `DirNode` listing them; cleanup
    /// is the caller's, matching every other `scratch_data_home`-style
    /// test in this file/crate.
    fn thumbnail_phase_fixture(label: &str, n: usize) -> (PathBuf, PathBuf, DirNode) {
        let data_home = std::env::temp_dir().join(format!(
            "arama-t061-thumbnail-phase-{}-{label}-data",
            std::process::id()
        ));
        let source_dir = std::env::temp_dir().join(format!(
            "arama-t061-thumbnail-phase-{}-{label}-source",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&data_home);
        let _ = std::fs::remove_dir_all(&source_dir);
        std::fs::create_dir_all(&source_dir).unwrap();

        let files: Vec<PathBuf> = (0..n)
            .map(|i| {
                let path = source_dir.join(format!("image-{i:05}.jpg"));
                std::fs::write(&path, MINIMAL_JPEG).unwrap();
                path
            })
            .collect();

        (
            data_home,
            source_dir.clone(),
            DirNode {
                path: source_dir,
                sub_dirs: vec![],
                files,
            },
        )
    }

    #[test]
    fn run_thumbnail_phase_processes_every_file_when_never_cancelled() {
        let _guard = ARAMA_DATA_HOME_TEST_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let previous = std::env::var_os(DATA_HOME_ENV_VAR);
        let file_count = THUMBNAIL_CHUNK_SIZE * 3 + 7; // spans several chunks, not a round number of them
        let (data_home, source_dir, dir_node) = thumbnail_phase_fixture("full-run", file_count);
        unsafe {
            std::env::set_var(DATA_HOME_ENV_VAR, &data_home);
        }

        let runtime = tokio::runtime::Runtime::new().unwrap();
        let cancel_flag = Arc::new(AtomicBool::new(false));
        let results = runtime.block_on(run_thumbnail_phase(dir_node, cancel_flag));

        assert_eq!(
            results.len(),
            file_count,
            "an uncancelled run must process every file"
        );
        let failures: Vec<_> = results.iter().filter(|(_, r)| r.is_err()).collect();
        assert!(
            failures.is_empty(),
            "every real, decodable file must cache successfully: {failures:?}"
        );

        unsafe {
            match &previous {
                Some(value) => std::env::set_var(DATA_HOME_ENV_VAR, value),
                None => std::env::remove_var(DATA_HOME_ENV_VAR),
            }
        }
        let _ = std::fs::remove_dir_all(&data_home);
        let _ = std::fs::remove_dir_all(&source_dir);
    }

    /// The property RFC 047 S3.2 is actually about: setting the flag
    /// mid-run must leave *some* work undone (not every chunk already
    /// dispatched) and must return in time bounded by roughly one
    /// chunk's processing cost, not by finishing the whole directory.
    #[test]
    fn run_thumbnail_phase_stops_early_once_cancel_flag_is_set() {
        let _guard = ARAMA_DATA_HOME_TEST_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let previous = std::env::var_os(DATA_HOME_ENV_VAR);
        // Enough chunks that cancelling partway through is observable:
        // too few and the whole run could finish before the canceller
        // task is even scheduled, which would prove nothing either way.
        let file_count = THUMBNAIL_CHUNK_SIZE * 40;
        let (data_home, source_dir, dir_node) =
            thumbnail_phase_fixture("cancel-mid-run", file_count);
        unsafe {
            std::env::set_var(DATA_HOME_ENV_VAR, &data_home);
        }

        let runtime = tokio::runtime::Runtime::new().unwrap();
        let cancel_flag = Arc::new(AtomicBool::new(false));
        let canceller_flag = cancel_flag.clone();

        let started = Instant::now();
        let results = runtime.block_on(async move {
            let canceller = tokio::spawn(async move {
                tokio::time::sleep(Duration::from_millis(20)).await;
                canceller_flag.store(true, Ordering::Relaxed);
            });
            let results = run_thumbnail_phase(dir_node, cancel_flag).await;
            let _ = canceller.await;
            results
        });
        let elapsed = started.elapsed();

        eprintln!(
            "=== T061 evidence: cancel set after 20ms, {} of {file_count} files \
             processed, run_thumbnail_phase returned after {elapsed:?} ===",
            results.len()
        );

        assert!(
            results.len() < file_count,
            "expected the cancel flag to stop the run before every chunk ran, got all \
             {file_count} files processed"
        );
        assert!(
            !results.is_empty(),
            "the chunk(s) already dispatched before cancellation should still have \
             produced some results, got none at all"
        );
        assert!(
            elapsed < Duration::from_secs(5),
            "a cancelled run must return in roughly one chunk's worth of time, not finish \
             the whole directory - took {elapsed:?} for {file_count} files"
        );

        unsafe {
            match &previous {
                Some(value) => std::env::set_var(DATA_HOME_ENV_VAR, value),
                None => std::env::remove_var(DATA_HOME_ENV_VAR),
            }
        }
        let _ = std::fs::remove_dir_all(&data_home);
        let _ = std::fs::remove_dir_all(&source_dir);
    }

    /// Cancelling *before* the first chunk ever dispatches must return
    /// an empty result, not panic or hang - the same shape
    /// `handle_cache_require`'s `!ret.is_empty()` check downstream
    /// depends on to correctly skip the embedding phase.
    #[test]
    fn run_thumbnail_phase_returns_empty_when_already_cancelled_before_starting() {
        let _guard = ARAMA_DATA_HOME_TEST_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let previous = std::env::var_os(DATA_HOME_ENV_VAR);
        let (data_home, source_dir, dir_node) =
            thumbnail_phase_fixture("pre-cancelled", THUMBNAIL_CHUNK_SIZE * 2);
        unsafe {
            std::env::set_var(DATA_HOME_ENV_VAR, &data_home);
        }

        let runtime = tokio::runtime::Runtime::new().unwrap();
        let cancel_flag = Arc::new(AtomicBool::new(true));
        let results = runtime.block_on(run_thumbnail_phase(dir_node, cancel_flag));

        assert!(
            results.is_empty(),
            "a flag already set before the first chunk must produce no results"
        );

        unsafe {
            match &previous {
                Some(value) => std::env::set_var(DATA_HOME_ENV_VAR, value),
                None => std::env::remove_var(DATA_HOME_ENV_VAR),
            }
        }
        let _ = std::fs::remove_dir_all(&data_home);
        let _ = std::fs::remove_dir_all(&source_dir);
    }
}
