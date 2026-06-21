//! One-time import of a v1 (`file-feature-cache`) database into the v2
//! (`localcache`) database. See RFC 002, § Data migration.
//!
//! The importer is **read-only on the v1 file**. For every v1 row it:
//!
//! 1. Skips files that no longer exist on disk, or whose `mtime` (in
//!    nanoseconds, as v1 stored it) no longer matches — the payload is
//!    stale and will be recomputed lazily.
//! 2. Moves the v1 thumbnail (named `<row id>.jpg`) to its v2 name
//!    (`<blake3(path)[..16]>.jpg`) in the same directory.
//! 3. Writes the payload into the matching namespace: rows with a
//!    `video_features` record become video entries; everything else
//!    becomes an image entry (matching v1 usage, where the startup
//!    thumbnail pass registered every media file through the image
//!    writer).
//!
//! On success the v1 file is renamed to `<name>.v1.bak`; on failure the
//! partially-written v2 file is removed so the next run can retry.
//!
//! This module is scheduled for removal one release cycle after v2 ships.

use std::path::Path;

use localcache::ConnectionPool;

use crate::core::engine::{
    CacheConfig, CacheError, DbLocation, IMAGE_PAYLOAD_VERSION, NAMESPACE_IMAGE, NAMESPACE_VIDEO,
    Result, VIDEO_PAYLOAD_VERSION, cache_options,
};
use crate::core::payload::{ImagePayload, VideoPayload};
use crate::core::thumbnail::thumbnail_dest_for_canonical;

/// Outcome of a migration run.
#[derive(Debug, Default)]
pub struct MigrationReport {
    /// Entries imported into the v2 database.
    pub imported: usize,
    /// Entries skipped (file gone, changed, or unreadable payload).
    pub skipped: usize,
}

/// Import `v1_db` into `v2_db` when, and only when, the former exists and
/// the latter does not. Returns `Ok(None)` when there is nothing to do.
pub fn migrate_v1_if_present(v1_db: &Path, v2_db: &Path) -> Result<Option<MigrationReport>> {
    if !v1_db.exists() || v2_db.exists() {
        return Ok(None);
    }

    match import(v1_db, v2_db) {
        Ok(report) => {
            // Keep the v1 file around for one release cycle as a backup.
            let backup = v1_db.with_extension("sqlite.v1.bak");
            let _ = std::fs::rename(v1_db, backup);
            Ok(Some(report))
        }
        Err(err) => {
            // Leave the v1 file untouched and remove the partial v2 file
            // so that the next startup can retry (or fall back to lazy
            // recomputation once the v2 file is created normally).
            let _ = std::fs::remove_file(v2_db);
            Err(err)
        }
    }
}

fn import(v1_db: &Path, v2_db: &Path) -> Result<MigrationReport> {
    let conn =
        rusqlite::Connection::open_with_flags(v1_db, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
            .map_err(|e| CacheError::Migration(e.to_string()))?;

    let config = CacheConfig {
        db_location: DbLocation::Custom(v2_db.to_path_buf()),
        ..CacheConfig::default()
    };
    let image_pool: ConnectionPool<ImagePayload> = ConnectionPool::open(cache_options(
        &config,
        NAMESPACE_IMAGE,
        IMAGE_PAYLOAD_VERSION,
    ))?;
    let video_pool: ConnectionPool<VideoPayload> = ConnectionPool::open(cache_options(
        &config,
        NAMESPACE_VIDEO,
        VIDEO_PAYLOAD_VERSION,
    ))?;

    let mut report = MigrationReport::default();

    let mut stmt = conn
        .prepare(
            "SELECT f.id, f.path, f.mtime_ns,
                    t.thumbnail_path,
                    i.clip_vector,
                    v.clip_vector, v.wav2vec2_vector
             FROM files f
             LEFT JOIN thumbnails     t ON t.id = f.id
             LEFT JOIN image_features i ON i.id = f.id
             LEFT JOIN video_features v ON v.id = f.id",
        )
        .map_err(|e| CacheError::Migration(e.to_string()))?;

    let rows = stmt
        .query_map([], |r| {
            Ok(V1Row {
                path: r.get::<_, String>(1)?,
                mtime_ns: r.get::<_, Option<i64>>(2)?,
                thumbnail_path: r.get::<_, Option<String>>(3)?,
                image_clip: r.get::<_, Option<Vec<u8>>>(4)?,
                video_clip: r.get::<_, Option<Vec<u8>>>(5)?,
                video_wav: r.get::<_, Option<Vec<u8>>>(6)?,
            })
        })
        .map_err(|e| CacheError::Migration(e.to_string()))?;

    for row in rows {
        let row = row.map_err(|e| CacheError::Migration(e.to_string()))?;

        if !still_fresh(&row) {
            report.skipped += 1;
            continue;
        }

        let thumbnail_path = relocate_thumbnail(&row);

        // Rows carrying a video_features record are video entries;
        // everything else is an image entry.
        let outcome = if row.video_clip.is_some() || row.video_wav.is_some() {
            video_pool.set(
                &row.path,
                &VideoPayload {
                    thumbnail_path,
                    clip_vector: row.video_clip.as_deref().map(blob_to_vec),
                    wav2vec2_vector: row.video_wav.as_deref().map(blob_to_vec),
                },
            )
        } else {
            image_pool.set(
                &row.path,
                &ImagePayload {
                    thumbnail_path,
                    clip_vector: row.image_clip.as_deref().map(blob_to_vec),
                },
            )
        };

        match outcome {
            Ok(()) => report.imported += 1,
            // A single bad file must not abort the whole migration.
            Err(_) => report.skipped += 1,
        }
    }

    Ok(report)
}

struct V1Row {
    path: String,
    mtime_ns: Option<i64>,
    thumbnail_path: Option<String>,
    image_clip: Option<Vec<u8>>,
    video_clip: Option<Vec<u8>>,
    video_wav: Option<Vec<u8>>,
}

/// The payload is only worth importing when the file still exists and
/// its mtime matches what v1 recorded (nanosecond precision). Changed
/// files are skipped and recomputed lazily.
fn still_fresh(row: &V1Row) -> bool {
    let Ok(meta) = std::fs::metadata(&row.path) else {
        return false;
    };
    let Some(stored_ns) = row.mtime_ns else {
        return false;
    };
    let current_ns = meta
        .modified()
        .ok()
        .and_then(|m| m.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_nanos() as i64);
    current_ns == Some(stored_ns)
}

/// Move the v1 thumbnail file (named by row id) to its v2 hash-based
/// name in the same directory. Returns the new path, or `None` when the
/// v1 thumbnail is absent on disk.
fn relocate_thumbnail(row: &V1Row) -> Option<String> {
    let old = Path::new(row.thumbnail_path.as_deref()?);
    if !old.exists() {
        return None;
    }
    let dir = old.parent()?;
    let new = thumbnail_dest_for_canonical(dir, &row.path);
    if new != old {
        std::fs::rename(old, &new).ok()?;
    }
    Some(new.to_string_lossy().into_owned())
}

/// Decode a v1 raw little-endian `f32` blob.
fn blob_to_vec(blob: &[u8]) -> Vec<f32> {
    blob.chunks_exact(4)
        .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect()
}
