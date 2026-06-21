//! `VideoCacheWriter` / `VideoCacheReader` — video-specific cache handles.
//!
//! Backed by `localcache` (RFC 002). Same architecture as the image
//! handles; video adds ffmpeg poster-thumbnail extraction and a second
//! feature vector (wav2vec2). `None` fields in an upsert request preserve
//! the stored value (the v1 SQL `COALESCE` semantics).

use std::path::{Path, PathBuf};
use std::sync::Arc;

use localcache::{CacheEntry, CacheStatus, ConnectionPool, ReadPool};
use rayon::prelude::*;

use crate::core::engine::{
    CacheConfig, DbLocation, NAMESPACE_VIDEO, Result, VIDEO_PAYLOAD_VERSION, cache_options,
    ensure_db_dir, ensure_schema, is_fresh, read_pool_size,
};
use crate::core::payload::VideoPayload;
use crate::core::thumbnail::{generate_video_thumbnail, thumbnail_dest};
use crate::types::{
    CacheRead, DirCacheSummary, LookupResult, UpsertVideoRequest, VideoCacheEntry, VideoFeatures,
};

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct VideoCacheConfig {
    pub cache_config: CacheConfig,
    /// Path of the ffmpeg executable. `None` skips thumbnail generation.
    pub ffmpeg_path: Option<PathBuf>,
}

// ---------------------------------------------------------------------------
// VideoCacheWriter
// ---------------------------------------------------------------------------

/// Update handle for video files.
///
/// - Generates poster thumbnails with ffmpeg (frame at 5 s, falling back
///   to 0 s).
/// - `Clone` only bumps `Arc` counters.
#[derive(Clone)]
pub struct VideoCacheWriter {
    write: ConnectionPool<VideoPayload>,
    read: ReadPool<VideoPayload>,
    config: Arc<VideoCacheConfig>,
}

impl VideoCacheWriter {
    pub fn as_session(config: VideoCacheConfig) -> Result<Self> {
        let options = cache_options(&config.cache_config, NAMESPACE_VIDEO, VIDEO_PAYLOAD_VERSION);
        // Create the parent directory before localcache touches SQLite.
        ensure_db_dir(&options)?;
        let write = ConnectionPool::open(options.clone())?;
        write.with(|e| e.purge_stale_versions())?;
        let read = ReadPool::open(options, read_pool_size(&config.cache_config))?;
        Ok(Self {
            write,
            read,
            config: Arc::new(config),
        })
    }

    pub fn onetime(
        location: DbLocation,
        thumbnail_dir: Option<PathBuf>,
        ffmpeg_path: Option<PathBuf>,
    ) -> Result<Self> {
        Self::as_session(VideoCacheConfig {
            cache_config: CacheConfig {
                db_location: location,
                thumbnail_dir,
                ..CacheConfig::default()
            },
            ffmpeg_path,
        })
    }

    // -----------------------------------------------------------------------
    // Update API
    // -----------------------------------------------------------------------

    pub fn upsert(&self, req: UpsertVideoRequest) -> Result<()> {
        let status = self.write.check_status(&req.path)?;
        let thumbnail = self.ensure_thumbnail(&req.path, &status)?;
        self.commit(&req, Prepared { status, thumbnail })
    }

    /// Batch variant of `upsert`. Returns `(PathBuf, Result<()>)` per
    /// request. Freshness checks and thumbnail extraction run in
    /// parallel; writes are serialized; fresh nothing-new entries are
    /// skipped without a write. Individual errors are stored per
    /// element; other requests continue.
    pub fn upsert_all(&self, reqs: Vec<UpsertVideoRequest>) -> Vec<(PathBuf, Result<()>)> {
        let prepared: Vec<(UpsertVideoRequest, Result<Prepared>)> = reqs
            .into_par_iter()
            .map(|req| {
                let prep = (|| {
                    let status = self.read.check_status(&req.path)?;
                    let thumbnail = self.ensure_thumbnail(&req.path, &status)?;
                    Ok(Prepared { status, thumbnail })
                })();
                (req, prep)
            })
            .collect();

        prepared
            .into_iter()
            .map(|(req, prep)| {
                let path = req.path.clone();
                let result = prep.and_then(|p| self.commit(&req, p));
                (path, result)
            })
            .collect()
    }

    pub fn delete(&self, path: &Path) -> Result<bool> {
        Ok(self.write.remove(path)?)
    }

    /// Remove every cached entry whose file lives directly in `dir`,
    /// deleting the associated thumbnail file (when recorded) as well.
    /// Non-recursive. Returns the number of entries removed (RFC 004).
    pub fn delete_in_dir(&self, dir: &Path) -> Result<usize> {
        let entries = self.read.query_run(|q| q.path_in_dir(dir, false))?;
        let mut removed = 0;
        for entry in entries {
            if let Some(thumb) = &entry.payload.thumbnail_path {
                let _ = std::fs::remove_file(thumb);
            }
            if self.write.remove(&entry.path)? {
                removed += 1;
            }
        }
        Ok(removed)
    }

    pub fn list_paths(&self) -> Result<Vec<String>> {
        let keys = self.write.with(|e| e.keys(None))?;
        Ok(keys
            .into_iter()
            .map(|p| p.to_string_lossy().into_owned())
            .collect())
    }

    pub fn as_reader(&self) -> VideoCacheReader {
        VideoCacheReader {
            read: self.read.clone(),
        }
    }

    pub fn lookup(&self, path: &Path) -> Result<LookupResult<VideoCacheEntry>> {
        self.as_reader().lookup(path)
    }

    // -----------------------------------------------------------------------
    // Internal
    // -----------------------------------------------------------------------

    /// Merge with the existing payload (when fresh) and write. `None`
    /// vectors in the request preserve stored values, matching the v1
    /// `COALESCE` update semantics.
    fn commit(&self, req: &UpsertVideoRequest, prep: Prepared) -> Result<()> {
        let existing = if is_fresh(&prep.status) {
            self.write.get(&req.path)?.map(|e| e.payload)
        } else {
            None
        };

        // Steady-state skip: fresh entry, no new vectors, thumbnail
        // already recorded.
        if is_fresh(&prep.status) && req.clip_vector.is_none() && req.wav2vec2_vector.is_none() {
            if let Some(p) = &existing {
                if prep.thumbnail.is_none() || p.thumbnail_path == prep.thumbnail {
                    return Ok(());
                }
            }
        }

        let mut payload = existing.unwrap_or_default();
        if let Some(t) = prep.thumbnail {
            payload.thumbnail_path = Some(t);
        }
        if let Some(v) = &req.clip_vector {
            payload.clip_vector = Some(v.clone());
        }
        if let Some(v) = &req.wav2vec2_vector {
            payload.wav2vec2_vector = Some(v.clone());
        }
        self.write.set(&req.path, &payload)?;
        Ok(())
    }

    /// Extract (or reuse) the poster thumbnail for `path`. Skipped when
    /// either `ffmpeg_path` or `thumbnail_dir` is unset. Regenerated when
    /// the file changed.
    fn ensure_thumbnail(&self, path: &Path, status: &CacheStatus) -> Result<Option<String>> {
        let (Some(ffmpeg), Some(thumb_dir)) = (
            &self.config.ffmpeg_path,
            &self.config.cache_config.thumbnail_dir,
        ) else {
            return Ok(None);
        };
        let dest = thumbnail_dest(thumb_dir, path)?;
        if !dest.exists() || !is_fresh(status) {
            generate_video_thumbnail(path, &dest, ffmpeg)?;
        }
        Ok(Some(dest.to_string_lossy().into_owned()))
    }
}

/// Intermediate result of the parallel preparation phase.
struct Prepared {
    status: CacheStatus,
    thumbnail: Option<String>,
}

// ---------------------------------------------------------------------------
// VideoCacheReader
// ---------------------------------------------------------------------------

/// Read-only handle for video files. `Clone` only bumps `Arc` counters;
/// clones share the same read pool and may be used from many threads.
#[derive(Clone)]
pub struct VideoCacheReader {
    read: ReadPool<VideoPayload>,
}

impl VideoCacheReader {
    pub fn as_session(config: VideoCacheConfig) -> Result<Self> {
        let options = cache_options(&config.cache_config, NAMESPACE_VIDEO, VIDEO_PAYLOAD_VERSION);
        // Create the parent directory before localcache touches SQLite.
        ensure_db_dir(&options)?;
        ensure_schema::<VideoPayload>(&options)?;
        let read = ReadPool::open(options, read_pool_size(&config.cache_config))?;
        Ok(Self { read })
    }

    pub fn onetime(location: DbLocation) -> Result<Self> {
        Self::as_session(VideoCacheConfig {
            cache_config: CacheConfig {
                db_location: location,
                ..CacheConfig::default()
            },
            ffmpeg_path: None,
        })
    }

    pub fn lookup(&self, path: &Path) -> Result<LookupResult<VideoCacheEntry>> {
        let canonical = match path.canonicalize() {
            Ok(p) => p,
            Err(_) => return Ok(LookupResult::Miss),
        };

        match self.read.check_status(&canonical)? {
            CacheStatus::Missing => Ok(LookupResult::Miss),
            CacheStatus::Stale => Ok(LookupResult::Invalidated),
            CacheStatus::Fresh => match self.read.get(&canonical)? {
                None => Ok(LookupResult::Miss),
                Some(entry) => Ok(LookupResult::Hit(to_video_entry(entry))),
            },
        }
    }

    /// Batch variant of `lookup`, parallelized with rayon over the
    /// read pool's connections.
    pub fn lookup_all(
        &self,
        paths: &[&Path],
    ) -> Vec<(PathBuf, Result<LookupResult<VideoCacheEntry>>)> {
        paths
            .par_iter()
            .map(|p| (p.to_path_buf(), self.lookup(p)))
            .collect()
    }

    pub fn check(&self, path: &Path) -> Result<bool> {
        Ok(is_fresh(&self.read.check_status(path)?))
    }

    pub fn list_paths(&self) -> Result<Vec<String>> {
        Ok(self
            .read
            .keys(None)?
            .into_iter()
            .map(|p| p.to_string_lossy().into_owned())
            .collect())
    }

    pub fn all(&self) -> Result<Vec<Result<VideoCacheEntry>>> {
        let entries = self.read.query_run(|q| q)?;
        Ok(entries.into_iter().map(|e| Ok(to_video_entry(e))).collect())
    }

    /// Group all cached entries by their parent directory and aggregate
    /// count, total size, and the newest cached-at timestamp (RFC 004).
    pub fn summarize_by_dir(&self) -> Result<Vec<DirCacheSummary>> {
        crate::core::image::summarize_entries(self.read.list_entries()?)
    }

    pub fn all_in_dir(&self, path: &Path) -> Result<Vec<Result<VideoCacheEntry>>> {
        let dir = dir_of(path);
        let entries = self.read.query_run(|q| q.path_in_dir(dir, false))?;
        Ok(entries.into_iter().map(|e| Ok(to_video_entry(e))).collect())
    }

    pub fn all_in_dir_and_sub_dirs(&self, path: &Path) -> Result<Vec<Result<VideoCacheEntry>>> {
        let dir = dir_of(path);
        let entries = self.read.query_run(|q| q.path_in_dir(dir, true))?;
        Ok(entries.into_iter().map(|e| Ok(to_video_entry(e))).collect())
    }
}

impl CacheRead for VideoCacheReader {
    fn check(&self, path: &Path) -> Result<bool> {
        VideoCacheReader::check(self, path)
    }

    fn check_all(&self, paths: &[&Path]) -> Vec<(PathBuf, Result<bool>)> {
        paths
            .par_iter()
            .map(|p| (p.to_path_buf(), self.check(p)))
            .collect()
    }

    fn list_paths(&self) -> Result<Vec<String>> {
        VideoCacheReader::list_paths(self)
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn dir_of(path: &Path) -> &Path {
    if path.is_dir() {
        path
    } else {
        path.parent().unwrap_or(path)
    }
}

// ---------------------------------------------------------------------------
// Mapping
// ---------------------------------------------------------------------------

fn to_video_entry(entry: CacheEntry<VideoPayload>) -> VideoCacheEntry {
    let has_features =
        entry.payload.clip_vector.is_some() || entry.payload.wav2vec2_vector.is_some();
    VideoCacheEntry {
        path: entry.path.to_string_lossy().into_owned(),
        thumbnail_path: entry.payload.thumbnail_path,
        features: has_features.then_some(VideoFeatures {
            clip_vector: entry.payload.clip_vector,
            wav2vec2_vector: entry.payload.wav2vec2_vector,
        }),
    }
}
