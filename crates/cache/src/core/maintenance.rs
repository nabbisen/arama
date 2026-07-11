//! Cache maintenance operations for storage footprint and pruning (RFC 016).

use std::collections::{BTreeSet, HashMap};
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

use localcache::{ConnectionPool, EntryInfo};

use crate::CacheError;
use crate::core::engine::{
    CacheConfig, DbLocation, IMAGE_PAYLOAD_VERSION, NAMESPACE_IMAGE, NAMESPACE_VIDEO, Result,
    VIDEO_PAYLOAD_VERSION, cache_options, ensure_db_dir,
};
use crate::core::payload::{ImagePayload, VideoPayload};
use crate::types::{CacheFootprint, CachePruneReport, CachePruneRequest};

#[derive(Clone)]
pub struct CacheMaintenance {
    config: CacheConfig,
}

impl CacheMaintenance {
    pub fn as_session(config: CacheConfig) -> Result<Self> {
        let options = cache_options(&config, NAMESPACE_IMAGE, IMAGE_PAYLOAD_VERSION);
        ensure_db_dir(&options)?;
        Ok(Self { config })
    }

    pub fn onetime(location: DbLocation, thumbnail_dir: Option<PathBuf>) -> Result<Self> {
        Self::as_session(CacheConfig {
            db_location: location,
            thumbnail_dir,
            ..CacheConfig::default()
        })
    }

    pub fn footprint(&self) -> Result<CacheFootprint> {
        footprint(&self.config)
    }

    pub fn prune(&self, request: CachePruneRequest) -> Result<CachePruneReport> {
        let before = self.footprint()?;
        if before.total_bytes <= request.max_bytes {
            return Ok(CachePruneReport {
                before,
                after: before,
                target_reached: true,
                ..CachePruneReport::default()
            });
        }

        let db_path = self.config.db_location.resolve();
        let mut referenced_thumbnails = BTreeSet::new();
        let mut candidates = Vec::new();

        let image_pool = if db_path.exists() {
            let options = cache_options(&self.config, NAMESPACE_IMAGE, IMAGE_PAYLOAD_VERSION);
            Some(ConnectionPool::<ImagePayload>::open(options)?)
        } else {
            None
        };
        let video_pool = if db_path.exists() {
            let options = cache_options(&self.config, NAMESPACE_VIDEO, VIDEO_PAYLOAD_VERSION);
            Some(ConnectionPool::<VideoPayload>::open(options)?)
        } else {
            None
        };

        if let Some(pool) = &image_pool {
            collect_image_candidates(pool, &mut referenced_thumbnails, &mut candidates)?;
        }
        if let Some(pool) = &video_pool {
            collect_video_candidates(pool, &mut referenced_thumbnails, &mut candidates)?;
        }

        let mut current_bytes = before.total_bytes;
        let removed_orphan_thumbnail_bytes = remove_orphan_thumbnails(
            self.config.thumbnail_dir.as_deref(),
            &referenced_thumbnails,
            &mut current_bytes,
        )?;

        let mut removed_entries = 0;
        let mut removed_recorded_thumbnail_bytes = 0;
        if request.max_bytes < current_bytes {
            candidates.sort_by(|a, b| {
                (a.updated_at, &a.path, a.namespace).cmp(&(b.updated_at, &b.path, b.namespace))
            });

            for candidate in candidates {
                if current_bytes <= request.max_bytes {
                    break;
                }
                if candidate.thumbnail_bytes == 0 {
                    continue;
                }

                let removed = match candidate.namespace {
                    Namespace::Image => image_pool
                        .as_ref()
                        .map(|pool| pool.remove(&candidate.path))
                        .transpose()?
                        .unwrap_or(false),
                    Namespace::Video => video_pool
                        .as_ref()
                        .map(|pool| pool.remove(&candidate.path))
                        .transpose()?
                        .unwrap_or(false),
                };

                if removed {
                    removed_entries += 1;
                    if let Some(thumbnail) = candidate.thumbnail_path {
                        let bytes = remove_file_counting_best_effort(&thumbnail);
                        removed_recorded_thumbnail_bytes += bytes;
                        current_bytes = current_bytes.saturating_sub(bytes);
                    }
                }
            }
        }

        let after = self.footprint()?;
        let target_reached = after.total_bytes <= request.max_bytes;
        let unreclaimable_bytes = if target_reached {
            0
        } else {
            after.total_bytes.saturating_sub(request.max_bytes)
        };

        Ok(CachePruneReport {
            before,
            after,
            target_reached,
            unreclaimable_bytes,
            removed_entries,
            removed_recorded_thumbnail_bytes,
            removed_orphan_thumbnail_bytes,
        })
    }
}

fn footprint(config: &CacheConfig) -> Result<CacheFootprint> {
    let db = config.db_location.resolve();
    let database_bytes = file_len_or_zero(&db)?;
    let database_sidecar_bytes = file_len_or_zero(&sqlite_sidecar(&db, "-wal"))?
        + file_len_or_zero(&sqlite_sidecar(&db, "-shm"))?;
    let thumbnail_bytes = match &config.thumbnail_dir {
        Some(path) => dir_len_or_zero(path)?,
        None => 0,
    };
    Ok(CacheFootprint {
        database_bytes,
        database_sidecar_bytes,
        thumbnail_bytes,
        total_bytes: database_bytes + database_sidecar_bytes + thumbnail_bytes,
    })
}

fn collect_image_candidates(
    pool: &ConnectionPool<ImagePayload>,
    referenced_thumbnails: &mut BTreeSet<PathBuf>,
    candidates: &mut Vec<PruneCandidate>,
) -> Result<()> {
    let infos = entry_info_by_path(pool.list_entries()?);
    for entry in pool.query_run(|q| q)? {
        let Some(info) = infos.get(&entry.path) else {
            continue;
        };
        let thumbnail_path = entry.payload.thumbnail_path.map(PathBuf::from);
        push_candidate(
            candidates,
            referenced_thumbnails,
            Namespace::Image,
            &entry.path,
            info,
            thumbnail_path,
        )?;
    }
    Ok(())
}

fn collect_video_candidates(
    pool: &ConnectionPool<VideoPayload>,
    referenced_thumbnails: &mut BTreeSet<PathBuf>,
    candidates: &mut Vec<PruneCandidate>,
) -> Result<()> {
    let infos = entry_info_by_path(pool.list_entries()?);
    for entry in pool.query_run(|q| q)? {
        let Some(info) = infos.get(&entry.path) else {
            continue;
        };
        let thumbnail_path = entry.payload.thumbnail_path.map(PathBuf::from);
        push_candidate(
            candidates,
            referenced_thumbnails,
            Namespace::Video,
            &entry.path,
            info,
            thumbnail_path,
        )?;
    }
    Ok(())
}

fn push_candidate(
    candidates: &mut Vec<PruneCandidate>,
    referenced_thumbnails: &mut BTreeSet<PathBuf>,
    namespace: Namespace,
    path: &Path,
    info: &EntryInfo,
    thumbnail_path: Option<PathBuf>,
) -> Result<()> {
    let thumbnail_bytes = match &thumbnail_path {
        Some(path) => {
            referenced_thumbnails.insert(normalize_existing_path(path));
            file_len_or_zero(path)?
        }
        None => 0,
    };
    candidates.push(PruneCandidate {
        namespace,
        path: path.to_path_buf(),
        updated_at: info.updated_at,
        thumbnail_path,
        thumbnail_bytes,
    });
    Ok(())
}

fn entry_info_by_path(entries: Vec<EntryInfo>) -> HashMap<PathBuf, EntryInfo> {
    entries
        .into_iter()
        .map(|entry| (entry.path.clone(), entry))
        .collect()
}

fn remove_orphan_thumbnails(
    thumbnail_dir: Option<&Path>,
    referenced: &BTreeSet<PathBuf>,
    current_bytes: &mut u64,
) -> Result<u64> {
    let Some(thumbnail_dir) = thumbnail_dir else {
        return Ok(0);
    };
    let files = thumbnail_files(thumbnail_dir)?;
    let mut removed = 0;
    for file in files {
        if referenced.contains(&normalize_existing_path(&file)) {
            continue;
        }
        let bytes = remove_file_counting(&file)?;
        removed += bytes;
        *current_bytes = current_bytes.saturating_sub(bytes);
    }
    Ok(removed)
}

fn thumbnail_files(dir: &Path) -> Result<Vec<PathBuf>> {
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut files = Vec::new();
    collect_files(dir, &mut files)?;
    Ok(files)
}

fn collect_files(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(dir).map_err(|err| CacheError::io(dir, err))? {
        let entry = entry.map_err(|err| CacheError::io(dir, err))?;
        let path = entry.path();
        let metadata = entry
            .metadata()
            .map_err(|err| CacheError::io(path.as_path(), err))?;
        if metadata.is_dir() {
            collect_files(&path, files)?;
        } else if metadata.is_file() {
            files.push(path);
        }
    }
    Ok(())
}

fn file_len_or_zero(path: &Path) -> Result<u64> {
    match fs::metadata(path) {
        Ok(metadata) if metadata.is_file() => Ok(metadata.len()),
        Ok(_) => Ok(0),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(0),
        Err(err) => Err(CacheError::io(path, err)),
    }
}

fn dir_len_or_zero(path: &Path) -> Result<u64> {
    if !path.exists() {
        return Ok(0);
    }
    Ok(thumbnail_files(path)?
        .into_iter()
        .map(|path| file_len_or_zero(&path))
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .sum())
}

fn remove_file_counting(path: &Path) -> Result<u64> {
    let bytes = file_len_or_zero(path)?;
    match fs::remove_file(path) {
        Ok(()) => Ok(bytes),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(0),
        Err(err) => Err(CacheError::io(path, err)),
    }
}

fn remove_file_counting_best_effort(path: &Path) -> u64 {
    let Ok(bytes) = file_len_or_zero(path) else {
        return 0;
    };
    match fs::remove_file(path) {
        Ok(()) => bytes,
        Err(_) => 0,
    }
}

fn normalize_existing_path(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

fn sqlite_sidecar(path: &Path, suffix: &str) -> PathBuf {
    let mut os: OsString = path.as_os_str().to_os_string();
    os.push(suffix);
    PathBuf::from(os)
}

#[derive(Debug, Clone)]
struct PruneCandidate {
    namespace: Namespace,
    path: PathBuf,
    updated_at: i64,
    thumbnail_path: Option<PathBuf>,
    thumbnail_bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Namespace {
    Image,
    Video,
}
