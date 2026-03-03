//! サムネイル生成ロジック。
//!
//! - 画像: `image` クレートでリサイズ (224×224 JPEG)
//! - 動画: ffmpeg で 5 秒時点のフレームを抽出。失敗時は 0 秒にフォールバック。

use std::path::{Path, PathBuf};
use std::process::Command;

use file_feature_cache::error::{CacheError, Result};

/// CLIP モデルの標準入力サイズ
const THUMBNAIL_SIZE: u32 = 224;

// ---------------------------------------------------------------------------
// 画像サムネイル
// ---------------------------------------------------------------------------

/// 画像ファイルからサムネイルを生成して `dest` に保存する。
pub fn generate_image_thumbnail(src: &Path, dest: &Path) -> Result<()> {
    let img = image::open(src).map_err(|e| CacheError::ThumbnailGenerationFailed(e.to_string()))?;

    let thumb = img.resize(
        THUMBNAIL_SIZE,
        THUMBNAIL_SIZE,
        image::imageops::FilterType::Lanczos3,
    );

    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent).map_err(|e| CacheError::Io {
            path: parent.to_string_lossy().into_owned(),
            source: e,
        })?;
    }

    thumb
        .save(dest)
        .map_err(|e| CacheError::ThumbnailGenerationFailed(e.to_string()))?;
    Ok(())
}

// ---------------------------------------------------------------------------
// 動画サムネイル
// ---------------------------------------------------------------------------

/// 動画ファイルからサムネイルを生成して `dest` に保存する。
///
/// 5 秒時点のフレームを試み、失敗した場合は 0 秒にフォールバックする。
/// どちらも失敗した場合は `Err` を返す。
pub fn generate_video_thumbnail(src: &Path, dest: &Path, ffmpeg_path: &Path) -> Result<()> {
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent).map_err(|e| CacheError::Io {
            path: parent.to_string_lossy().into_owned(),
            source: e,
        })?;
    }

    // 5 秒時点を試みる
    if run_ffmpeg(ffmpeg_path, src, dest, "00:00:05").is_ok() && dest.exists() {
        return Ok(());
    }

    // 0 秒にフォールバック
    if run_ffmpeg(ffmpeg_path, src, dest, "00:00:00").is_ok() && dest.exists() {
        return Ok(());
    }

    Err(CacheError::ThumbnailGenerationFailed(format!(
        "ffmpeg failed to extract frame from {}",
        src.display()
    )))
}

fn run_ffmpeg(ffmpeg_path: &Path, src: &Path, dest: &Path, timestamp: &str) -> Result<()> {
    let status = Command::new(ffmpeg_path)
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
            "-y", // 上書き許可
            dest.to_str().unwrap_or(""),
        ])
        .output()
        .map_err(|e| CacheError::ThumbnailGenerationFailed(e.to_string()))?;

    if status.status.success() {
        Ok(())
    } else {
        Err(CacheError::ThumbnailGenerationFailed(
            String::from_utf8_lossy(&status.stderr).to_string(),
        ))
    }
}

// ---------------------------------------------------------------------------
// 共通ユーティリティ
// ---------------------------------------------------------------------------

/// サムネイルの保存先パスを決定する。
/// `<thumbnail_dir>/<file_id>.jpg`
pub fn thumbnail_dest(thumbnail_dir: &Path, file_id: i64) -> PathBuf {
    thumbnail_dir.join(format!("{file_id}.jpg"))
}
