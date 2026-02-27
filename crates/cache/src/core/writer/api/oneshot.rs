//! 単発呼び出し用の convenience API。
//!
//! `CacheWriter` を都度生成して操作する薄いラッパー。
//! 呼び出しのたびにコネクションプールを開閉するため、
//! **rayon 並行処理やホットパスには使わないこと**。
//! そのような用途では [`CacheWriter`] を `Clone` して使い回す
//! primary API を選択すること。
//!
//! # 使い分けの目安
//!
//! | 用途 | 推奨 API |
//! |---|---|
//! | 単発スクリプト・初期化処理 | convenience API (本モジュール) |
//! | rayon 並列処理・繰り返し呼び出し | [`CacheWriter`] |
//!
//! [`CacheWriter`]: crate::CacheWriter

use crate::core::writer::CacheWriter;
use crate::error::Result;
use crate::types::{UpsertImageRequest, UpsertVideoRequest};

// ---------------------------------------------------------------------------
// 更新
// ---------------------------------------------------------------------------

/// 画像ファイルのキャッシュを単発で登録 / 更新する。
///
/// 繰り返し呼ぶ場合は [`CacheWriter::upsert_image`] を使うこと。
pub fn upsert_image(req: UpsertImageRequest) -> Result<()> {
    CacheWriter::open()?.upsert_image(req)
}

/// 動画ファイルのキャッシュを単発で登録 / 更新する。
///
/// 繰り返し呼ぶ場合は [`CacheWriter::upsert_video`] を使うこと。
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
