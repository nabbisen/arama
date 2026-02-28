//! 単発更新 API。
//!
//! 呼び出しのたびに DB を開いて操作して閉じる薄いラッパー。
//! 繰り返し呼び出しや rayon 並列処理には [`session::CacheWriter`] を使うこと。
//!
//! [`session::CacheWriter`]: super::session::CacheWriter

use crate::core::writer::cache_writer::CacheWriter;
use crate::error::Result;
use crate::types::{UpsertImageRequest, UpsertVideoRequest};

/// 画像ファイルのキャッシュを単発で登録 / 更新する。
pub fn upsert_image(req: UpsertImageRequest) -> Result<()> {
    CacheWriter::open()?.upsert_image(req)
}

/// 動画ファイルのキャッシュを単発で登録 / 更新する。
pub fn upsert_video(req: UpsertVideoRequest) -> Result<()> {
    CacheWriter::open()?.upsert_video(req)
}

/// ファイルパスに紐付くキャッシュを単発で削除する。
pub fn delete(file_path: &str) -> Result<bool> {
    CacheWriter::open()?.delete(file_path)
}

/// ファイルの現在の状態を確認し、変更されていれば DB から削除して `false` を返す。
pub fn verify_or_invalidate(file_path: &str) -> Result<bool> {
    CacheWriter::open()?.verify_or_invalidate(file_path)
}
