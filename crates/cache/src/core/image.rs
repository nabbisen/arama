//! `ImageCacheWriter` / `ImageCacheReader` — image-specific cache handles.
//!
//! Backed by `localcache` (RFC 002): a [`ConnectionPool`] serializes
//! writes; a [`ReadPool`] of `read_conns` read-only connections serves
//! parallel lookups. Both are `Arc`-based, so `Clone` is cheap.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use localcache::{CacheEntry, CacheStatus, ConnectionPool, ReadPool};
use rayon::prelude::*;

use crate::core::engine::{
    CacheConfig, DbLocation, IMAGE_PAYLOAD_VERSION, NAMESPACE_IMAGE, Result, cache_options,
    ensure_db_dir, ensure_schema, is_fresh, read_pool_size,
};
use crate::core::payload::ImagePayload;
use crate::core::thumbnail::{generate_image_thumbnail, thumbnail_dest};
use crate::types::{CacheRead, ImageCacheEntry, ImageFeatures, LookupResult, UpsertImageRequest};

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct ImageCacheConfig {
    pub cache_config: CacheConfig,
}

// ---------------------------------------------------------------------------
// ImageCacheWriter
// ---------------------------------------------------------------------------

/// Update handle for image files.
///
/// - Generates thumbnails with the `image` crate (224×224 JPEG).
/// - `Clone` only bumps `Arc` counters.
#[derive(Clone)]
pub struct ImageCacheWriter {
    write: ConnectionPool<ImagePayload>,
    read: ReadPool<ImagePayload>,
    config: Arc<ImageCacheConfig>,
}

impl ImageCacheWriter {
    pub fn as_session(config: ImageCacheConfig) -> Result<Self> {
        let options = cache_options(
            &config.cache_config,
            NAMESPACE_IMAGE,
            IMAGE_PAYLOAD_VERSION,
        );
        // Create the parent directory before localcache touches SQLite.
        ensure_db_dir(&options)?;
        // The writable engine is opened first: it creates the database
        // file and schema, which the read-only pool cannot.
        let write = ConnectionPool::open(options.clone())?;
        // Entries written by an older pipeline version are dead weight.
        write.with(|e| e.purge_stale_versions())?;
        let read = ReadPool::open(options, read_pool_size(&config.cache_config))?;
        Ok(Self {
            write,
            read,
            config: Arc::new(config),
        })
    }

    pub fn onetime(location: DbLocation) -> Result<Self> {
        Self::as_session(ImageCacheConfig {
            cache_config: CacheConfig {
                db_location: location,
                ..CacheConfig::default()
            },
        })
    }

    // -----------------------------------------------------------------------
    // Update API
    // -----------------------------------------------------------------------

    pub fn upsert(&self, req: UpsertImageRequest) -> Result<()> {
        let status = self.write.check_status(&req.path)?;
        self.write_payload(&req, status)
    }

    /// Batch variant of `upsert`. Returns `(PathBuf, Result<()>)` per
    /// request.
    ///
    /// ## Parallelization strategy
    ///
    /// - **Freshness checks and thumbnail generation** run in parallel
    ///   (rayon over the read pool).
    /// - **Database writes** are serialized on the write connection.
    /// - Entries that are already fresh and carry nothing new are
    ///   skipped entirely — the steady-state startup pass over an
    ///   unchanged library performs no writes and no hashing.
    ///
    /// Individual errors are stored per element; other requests continue.
    pub fn upsert_all(&self, reqs: Vec<UpsertImageRequest>) -> Vec<(PathBuf, Result<()>)> {
        // Phase 1 (parallel): freshness + thumbnail generation.
        let prepared: Vec<(UpsertImageRequest, Result<Prepared>)> = reqs
            .into_par_iter()
            .map(|req| {
                let prep = self.prepare(&req);
                (req, prep)
            })
            .collect();

        // Phase 2 (serial): payload merge + write.
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

    pub fn list_paths(&self) -> Result<Vec<String>> {
        let keys = self.write.with(|e| e.keys(None))?;
        Ok(keys
            .into_iter()
            .map(|p| p.to_string_lossy().into_owned())
            .collect())
    }

    pub fn as_reader(&self) -> ImageCacheReader {
        ImageCacheReader {
            read: self.read.clone(),
        }
    }

    pub fn lookup(&self, path: &Path) -> Result<LookupResult<ImageCacheEntry>> {
        self.as_reader().lookup(path)
    }

    // -----------------------------------------------------------------------
    // Internal
    // -----------------------------------------------------------------------

    /// Parallel-safe preparation: freshness check (read pool) and
    /// thumbnail generation. No write-connection access.
    fn prepare(&self, req: &UpsertImageRequest) -> Result<Prepared> {
        let status = self.read.check_status(&req.path)?;
        let thumbnail = self.ensure_thumbnail(&req.path, &status)?;
        Ok(Prepared { status, thumbnail })
    }

    /// Serial commit: merge with the existing payload (when fresh) and
    /// write. Skips the write entirely in the fresh, nothing-new case.
    fn commit(&self, req: &UpsertImageRequest, prep: Prepared) -> Result<()> {
        let existing = if is_fresh(&prep.status) {
            self.write.get(&req.path)?.map(|e| e.payload)
        } else {
            None
        };

        // Steady-state skip: fresh entry, no new vector, thumbnail
        // already recorded.
        if is_fresh(&prep.status) && req.clip_vector.is_none() {
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
        self.write.set(&req.path, &payload)?;
        Ok(())
    }

    /// Single-upsert path: thumbnail + merge + write in one call.
    fn write_payload(&self, req: &UpsertImageRequest, status: CacheStatus) -> Result<()> {
        let thumbnail = self.ensure_thumbnail(&req.path, &status)?;
        self.commit(req, Prepared { status, thumbnail })
    }

    /// Generate (or reuse) the thumbnail for `path`, returning its
    /// destination path string. A thumbnail is regenerated when the file
    /// changed (`status != Fresh`), fixing the v1 behaviour of serving a
    /// stale thumbnail after the source was modified.
    fn ensure_thumbnail(&self, path: &Path, status: &CacheStatus) -> Result<Option<String>> {
        let Some(thumb_dir) = &self.config.cache_config.thumbnail_dir else {
            return Ok(None);
        };
        let dest = thumbnail_dest(thumb_dir, path)?;
        if !dest.exists() || !is_fresh(status) {
            generate_image_thumbnail(path, &dest)?;
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
// ImageCacheReader
// ---------------------------------------------------------------------------

/// Read-only handle for image files. `Clone` only bumps `Arc` counters;
/// clones share the same read pool and may be used from many threads.
#[derive(Clone)]
pub struct ImageCacheReader {
    read: ReadPool<ImagePayload>,
}

impl ImageCacheReader {
    pub fn as_session(config: ImageCacheConfig) -> Result<Self> {
        let options = cache_options(
            &config.cache_config,
            NAMESPACE_IMAGE,
            IMAGE_PAYLOAD_VERSION,
        );
        // Create the parent directory before localcache touches SQLite.
        ensure_db_dir(&options)?;
        // A standalone reader may be the first handle to ever touch this
        // database; make sure the schema exists before going read-only.
        ensure_schema::<ImagePayload>(&options)?;
        let read = ReadPool::open(options, read_pool_size(&config.cache_config))?;
        Ok(Self { read })
    }

    pub fn onetime(location: DbLocation) -> Result<Self> {
        Self::as_session(ImageCacheConfig {
            cache_config: CacheConfig {
                db_location: location,
                ..CacheConfig::default()
            },
        })
    }

    pub fn lookup(&self, path: &Path) -> Result<LookupResult<ImageCacheEntry>> {
        let canonical = match path.canonicalize() {
            Ok(p) => p,
            Err(_) => return Ok(LookupResult::Miss),
        };

        match self.read.check_status(&canonical)? {
            CacheStatus::Missing => Ok(LookupResult::Miss),
            CacheStatus::Stale => Ok(LookupResult::Invalidated),
            CacheStatus::Fresh => match self.read.get(&canonical)? {
                None => Ok(LookupResult::Miss),
                Some(entry) => Ok(LookupResult::Hit(to_image_entry(entry))),
            },
        }
    }

    /// Batch variant of `lookup`, parallelized with rayon over the
    /// read pool's connections.
    pub fn lookup_all(
        &self,
        paths: &[&Path],
    ) -> Vec<(PathBuf, Result<LookupResult<ImageCacheEntry>>)> {
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

    pub fn all(&self) -> Result<Vec<Result<ImageCacheEntry>>> {
        let entries = self.read.query_run(|q| q)?;
        Ok(entries.into_iter().map(|e| Ok(to_image_entry(e))).collect())
    }

    /// Return all entries whose file lives directly inside `path`.
    ///
    /// If `path` is a file rather than a directory (the common call-site
    /// pattern: "find all entries in the same directory as this file"),
    /// its parent directory is used automatically.
    pub fn all_in_dir(&self, path: &Path) -> Result<Vec<Result<ImageCacheEntry>>> {
        let dir = dir_of(path);
        let entries = self.read.query_run(|q| q.path_in_dir(dir, false))?;
        Ok(entries.into_iter().map(|e| Ok(to_image_entry(e))).collect())
    }

    /// Return all entries whose file lives anywhere under `path`
    /// (recursively).
    ///
    /// If `path` is a file, its parent directory is used automatically.
    pub fn all_in_dir_and_sub_dirs(&self, path: &Path) -> Result<Vec<Result<ImageCacheEntry>>> {
        let dir = dir_of(path);
        let entries = self.read.query_run(|q| q.path_in_dir(dir, true))?;
        Ok(entries.into_iter().map(|e| Ok(to_image_entry(e))).collect())
    }
}

impl CacheRead for ImageCacheReader {
    fn check(&self, path: &Path) -> Result<bool> {
        ImageCacheReader::check(self, path)
    }

    fn check_all(&self, paths: &[&Path]) -> Vec<(PathBuf, Result<bool>)> {
        paths
            .par_iter()
            .map(|p| (p.to_path_buf(), self.check(p)))
            .collect()
    }

    fn list_paths(&self) -> Result<Vec<String>> {
        ImageCacheReader::list_paths(self)
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Return `path` itself when it is a directory; otherwise return its parent.
///
/// Call sites that pass a media *file* path to `all_in_dir` / `all_in_dir_and_sub_dirs`
/// expect to query the directory that *contains* the file.  `path_in_dir`
/// requires a directory, so we resolve automatically here.
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

fn to_image_entry(entry: CacheEntry<ImagePayload>) -> ImageCacheEntry {
    ImageCacheEntry {
        path: entry.path.to_string_lossy().into_owned(),
        thumbnail_path: entry.payload.thumbnail_path,
        features: entry
            .payload
            .clip_vector
            .map(|v| ImageFeatures { clip_vector: v }),
    }
}
