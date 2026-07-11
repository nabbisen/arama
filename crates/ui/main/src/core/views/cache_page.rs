//! Cache control page (RFC 004).
//!
//! Shows a per-directory table of cached entries (file count, total
//! size, newest cached-at timestamp), with substring filtering,
//! per-row clearing, and a form to cache an arbitrary directory.

use std::path::PathBuf;

use arama_cache::{
    CacheConfig, CacheFootprint, CacheMaintenance, CachePruneReport, DbLocation, DirCacheSummary,
    ImageCacheReader, VideoCacheReader,
};
use arama_env::{cache_storage_path, cache_thumbnail_dir_path};
use iced::Task;

pub mod message;
mod update;
mod view;

use message::{Internal, Message};

/// One table row: per-directory aggregate, image + video merged.
#[derive(Debug, Clone)]
pub struct DirRow {
    pub dir_path: String,
    pub file_count: usize,
    pub total_size: u64,
    pub latest_cached_at: i64,
}

/// Loaded Cache page data: table rows plus actual storage footprint.
#[derive(Debug, Clone, Default)]
pub struct CacheLoad {
    pub rows: Vec<DirRow>,
    pub footprint: Option<CacheFootprint>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheLoadError {
    pub message: String,
}

#[derive(Debug, Clone, Default)]
pub struct CachePage {
    /// Merged per-directory rows, sorted newest-first.
    rows: Vec<DirRow>,
    /// Substring filter (case-insensitive). Empty shows all rows.
    filter: String,
    /// Path input of the add-directory form.
    dir_input: String,
    /// One-off prune target in MiB.
    prune_target_input: String,
    /// Actual cache footprint from the last page reload.
    footprint: Option<CacheFootprint>,
    /// Most recent table/footprint reload failure.
    load_error: Option<String>,
    /// Result of the most recent prune operation.
    last_prune_report: Option<CachePruneReport>,
    /// Directory of the active caching run, when one is in flight.
    active_run: Option<PathBuf>,
    /// True while a table reload is in flight.
    busy: bool,
    /// True while a manual prune operation is in flight.
    prune_busy: bool,
    /// True once the page has loaded at least once; reload hooks in the
    /// app only fire after the first visit.
    loaded: bool,
}

impl CachePage {
    /// Async table reload: read both namespaces, merge per directory.
    pub fn load_task(&mut self) -> Task<Message> {
        self.busy = true;
        Task::perform(
            async { load_rows().map_err(|message| CacheLoadError { message }) },
            |load| Message::Internal(Internal::RowsLoaded(load)),
        )
    }

    /// Mark the start of a caching run for `dir` (drives the ⏳ row
    /// indicator and disables clear/cache buttons). A placeholder row
    /// is inserted when the directory has no entries yet so it appears
    /// in the table immediately.
    pub fn run_started(&mut self, dir: PathBuf) {
        let canonical = dir.canonicalize().unwrap_or(dir);
        let key = canonical.to_string_lossy().into_owned();
        if !self.rows.iter().any(|r| r.dir_path == key) {
            self.rows.insert(
                0,
                DirRow {
                    dir_path: key,
                    file_count: 0,
                    total_size: 0,
                    latest_cached_at: 0,
                },
            );
        }
        self.active_run = Some(canonical);
    }

    /// Mark the end of the active caching run (if any).
    pub fn run_finished(&mut self) {
        self.active_run = None;
    }

    pub fn prune_finished(&mut self, report: Option<CachePruneReport>) {
        self.prune_busy = false;
        self.last_prune_report = report;
    }

    pub fn is_loaded(&self) -> bool {
        self.loaded
    }
}

pub(super) fn parse_mib_target(input: &str) -> Option<u64> {
    let value: f64 = input.trim().parse().ok()?;
    if !value.is_finite() || value <= 0.0 {
        return None;
    }
    Some((value * 1024.0 * 1024.0).round() as u64)
}

/// Load and merge per-directory summaries from both cache namespaces.
fn load_rows() -> Result<CacheLoad, String> {
    use std::collections::BTreeMap;

    let location = DbLocation::Custom(cache_storage_path().map_err(|err| err.to_string())?);
    let thumbnail_dir = cache_thumbnail_dir_path().map_err(|err| err.to_string())?;
    let config = CacheConfig {
        db_location: location.clone(),
        ..CacheConfig::default()
    };

    let image = ImageCacheReader::as_session(arama_cache::ImageCacheConfig {
        cache_config: config.clone(),
    })
    .map_err(|err| err.to_string())?
    .summarize_by_dir()
    .map_err(|err| err.to_string())?;
    let video = VideoCacheReader::as_session(arama_cache::VideoCacheConfig {
        cache_config: config,
        ffmpeg_path: None,
    })
    .map_err(|err| err.to_string())?
    .summarize_by_dir()
    .map_err(|err| err.to_string())?;

    // Merge: sum counts and sizes; keep the newest timestamp.
    let mut merged: BTreeMap<String, DirRow> = BTreeMap::new();
    for s in image.into_iter().chain(video) {
        merge_summary(&mut merged, s);
    }

    let mut rows: Vec<DirRow> = merged.into_values().collect();
    rows.sort_by_key(|row| std::cmp::Reverse(row.latest_cached_at));
    let footprint = CacheMaintenance::onetime(location, Some(thumbnail_dir))
        .map_err(|err| err.to_string())?
        .footprint()
        .map_err(|err| err.to_string())?;
    Ok(CacheLoad {
        rows,
        footprint: Some(footprint),
    })
}

fn merge_summary(map: &mut std::collections::BTreeMap<String, DirRow>, s: DirCacheSummary) {
    map.entry(s.dir_path.clone())
        .and_modify(|row| {
            row.file_count += s.file_count;
            row.total_size += s.total_size;
            row.latest_cached_at = row.latest_cached_at.max(s.latest_cached_at);
        })
        .or_insert(DirRow {
            dir_path: s.dir_path,
            file_count: s.file_count,
            total_size: s.total_size,
            latest_cached_at: s.latest_cached_at,
        });
}
