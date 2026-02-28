//! 単発呼び出し用の convenience API。
//!
//! `CacheReader` を都度生成して操作する薄いラッパー。
//! 呼び出しのたびにコネクションプールを開閉するため、
//! **rayon 並行処理やホットパスには使わないこと**。
//! そのような用途では [`CacheReader`] を `Clone` して使い回す
//! primary API を選択すること。
//!
//! # 使い分けの目安
//!
//! | 用途 | 推奨 API |
//! |---|---|
//! | 単発スクリプト・初期化処理 | convenience API (本モジュール) |
//! | rayon 並列処理・繰り返し呼び出し | [`CacheReader`] |
//!
//! [`CacheReader`]: crate::CacheReader

use std::path::Path;

use crate::core::writer::cache_writer::CacheWriter;
use crate::error::Result;
use crate::types::{ImageCacheEntry, LookupResult, VideoCacheEntry};

// ---------------------------------------------------------------------------
// 参照
// ---------------------------------------------------------------------------

/// 画像ファイルのキャッシュを単発で照会する。
pub fn lookup_image(path: &Path) -> Result<LookupResult<ImageCacheEntry>> {
    CacheWriter::open()?.lookup_image(path)
}

/// 動画ファイルのキャッシュを単発で照会する。
pub fn lookup_video(path: &Path) -> Result<LookupResult<VideoCacheEntry>> {
    CacheWriter::open()?.lookup_video(path)
}

/// 登録済みファイルパスの一覧を単発で取得する。
pub fn list_paths() -> Result<Vec<String>> {
    CacheWriter::open()?.list_paths()
}
