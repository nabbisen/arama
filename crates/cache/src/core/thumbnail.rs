//! Thumbnail generation.
//!
//! - **Images**: resized with the `image` crate (224×224 JPEG, Lanczos3).
//! - **Videos**: a frame extracted with the `ffmpeg` command at the
//!   5-second mark, falling back to 0 seconds on failure.
//!
//! Thumbnails are named by a BLAKE3 hash of the source file's canonical
//! path: `<thumbnail_dir>/<hash16>.jpg`. (v1 named them by database row
//! id; row ids no longer exist under `localcache`.)

use std::path::{Path, PathBuf};
use std::process::Command;

use super::engine::CacheError;

/// Standard CLIP model input size (px).
const THUMBNAIL_SIZE: u32 = 224;

/// Number of hex characters of the path hash used in the file name.
const THUMBNAIL_NAME_LEN: usize = 16;

// ---------------------------------------------------------------------------
// Image thumbnails
// ---------------------------------------------------------------------------

/// Generate a thumbnail from an image file and save it to `dest`.
///
/// - Format: JPEG
/// - Size: `THUMBNAIL_SIZE` × `THUMBNAIL_SIZE` (224 × 224)
/// - Resize filter: Lanczos3
pub(crate) fn generate_image_thumbnail(src: &Path, dest: &Path) -> Result<(), CacheError> {
    ensure_parent(dest)?;

    let img = image::open(src).map_err(|e| CacheError::ThumbnailGenerationFailed(e.to_string()))?;

    let thumb = img.resize(
        THUMBNAIL_SIZE,
        THUMBNAIL_SIZE,
        image::imageops::FilterType::Lanczos3,
    );

    thumb
        .save(dest)
        .map_err(|e| CacheError::ThumbnailGenerationFailed(e.to_string()))?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Video thumbnails
// ---------------------------------------------------------------------------

/// Generate a thumbnail from a video file and save it to `dest`.
///
/// Tries the frame at the 5-second mark; falls back to 0 seconds.
/// Returns `Err(CacheError::ThumbnailGenerationFailed)` when both fail.
pub(crate) fn generate_video_thumbnail(
    src: &Path,
    dest: &Path,
    ffmpeg_path: &Path,
) -> Result<(), CacheError> {
    ensure_parent(dest)?;

    // Try the 5-second mark first.
    if run_ffmpeg(ffmpeg_path, src, dest, "00:00:05").is_ok() && dest.exists() {
        return Ok(());
    }

    // Fall back to 0 seconds.
    if run_ffmpeg(ffmpeg_path, src, dest, "00:00:00").is_ok() && dest.exists() {
        return Ok(());
    }

    Err(CacheError::ThumbnailGenerationFailed(format!(
        "ffmpeg failed to extract frame from {}",
        src.display()
    )))
}

fn run_ffmpeg(
    ffmpeg_path: &Path,
    src: &Path,
    dest: &Path,
    timestamp: &str,
) -> Result<(), CacheError> {
    let output = Command::new(ffmpeg_path)
        .args([
            "-ss",
            timestamp,
            "-i",
            src.to_str().unwrap_or(""),
            "-vframes",
            "1",
            "-vf",
            &format!(
                "scale={THUMBNAIL_SIZE}:{THUMBNAIL_SIZE}:force_original_aspect_ratio=decrease"
            ),
            "-y", // overwrite an existing file
            dest.to_str().unwrap_or(""),
        ])
        .output()
        .map_err(|e| CacheError::ThumbnailGenerationFailed(e.to_string()))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(CacheError::ThumbnailGenerationFailed(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ))
    }
}

// ---------------------------------------------------------------------------
// Shared utilities
// ---------------------------------------------------------------------------

/// Thumbnail destination for a source file:
/// `<thumbnail_dir>/<blake3(canonical_path)[..16]>.jpg`.
///
/// The source file must exist (its path is canonicalized to make the name
/// stable across relative/absolute spellings).
pub(crate) fn thumbnail_dest(thumbnail_dir: &Path, src: &Path) -> Result<PathBuf, CacheError> {
    let canonical = src.canonicalize().map_err(|e| CacheError::io(src, e))?;
    Ok(thumbnail_dest_for_canonical(
        thumbnail_dir,
        &canonical.to_string_lossy(),
    ))
}

/// Same as [`thumbnail_dest`] for an already-canonicalized path string.
/// Used by the v1 migration, where the source file's existence has
/// already been verified.
pub(crate) fn thumbnail_dest_for_canonical(thumbnail_dir: &Path, canonical: &str) -> PathBuf {
    let hash = blake3::hash(canonical.as_bytes());
    let hex = hash.to_hex();
    thumbnail_dir.join(format!("{}.jpg", &hex.as_str()[..THUMBNAIL_NAME_LEN]))
}

/// Create the parent directory of `path` when missing.
fn ensure_parent(path: &Path) -> Result<(), CacheError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| CacheError::io(parent, e))?;
    }
    Ok(())
}
