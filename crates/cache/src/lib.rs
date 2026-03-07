//! # ai_cache
//!
//! 画像・動画 AI 推論結果 (サムネイル + 特徴量ベクトル) を SQLite にキャッシュする。
//!
//! [`file_feature_cache`] エンジンに依存し、画像・動画専用のスキーマ拡張と
//! サムネイル自動生成を提供する。
//!
//! ## Writer / Reader の選択
//!
//! | 型 | 用途 |
//! |---|---|
//! | [`ImageCacheWriter`] | 画像ファイルの登録・照会・削除 |
//! | [`ImageCacheReader`] | 画像ファイルの照会のみ (並列向け) |
//! | [`VideoCacheWriter`] | 動画ファイルの登録・照会・削除 |
//! | [`VideoCacheReader`] | 動画ファイルの照会のみ (並列向け) |
//!
//! ## 基本的な使い方
//!
//! ```rust,no_run
//! use ai_cache::{ImageCacheWriter, ImageCacheConfig, UpsertImageRequest, LookupResult};
//! use file_feature_cache::{CacheConfig, DbLocation};
//!
//! # fn main() -> anyhow::Result<()> {
//! let writer = ImageCacheWriter::as_session(ImageCacheConfig {
//!     cache_config: CacheConfig {
//!         db_location:   DbLocation::AppCache(None),
//!         read_conns:    4,
//!         thumbnail_dir: Some("/var/cache/myapp/thumbs".into()),
//!     },
//!     thumbnail: true,
//! })?;
//!
//! writer.upsert(UpsertImageRequest {
//!     file_path:   "/data/photo.jpg".to_string(),
//!     clip_vector: Some(vec![0.1, 0.2, 0.3]),
//! })?;
//!
//! match writer.lookup("/data/photo.jpg")? {
//!     LookupResult::Hit(entry) => {
//!         println!("thumbnail: {:?}", entry.thumbnail_path);
//!         println!("features:  {:?}", entry.features);
//!     }
//!     LookupResult::Invalidated => println!("file changed, cache cleared"),
//!     LookupResult::Miss        => println!("not cached"),
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## onetime — 単発呼び出し
//!
//! ```rust,no_run
//! use ai_cache::{ImageCacheWriter, LookupResult};
//! use file_feature_cache::DbLocation;
//!
//! # fn main() -> anyhow::Result<()> {
//! let result = ImageCacheWriter::onetime(DbLocation::WorkDir(None))?
//!     .lookup("/data/photo.jpg")?;
//! # Ok(())
//! # }
//! ```
//!
//! ## as_session + as_reader — スレッド並列処理
//!
//! ```rust,no_run
//! use ai_cache::{ImageCacheWriter, ImageCacheConfig};
//! use file_feature_cache::{CacheConfig, DbLocation};
//!
//! # fn main() -> anyhow::Result<()> {
//! let writer = ImageCacheWriter::as_session(ImageCacheConfig {
//!     cache_config: CacheConfig { db_location: DbLocation::WorkDir(None), read_conns: 8, thumbnail_dir: None },
//!     thumbnail: false,
//! })?;
//! let reader = writer.as_reader();
//!
//! std::thread::scope(|s| {
//!     for path in ["/data/a.jpg", "/data/b.jpg"] {
//!         let r = reader.clone();
//!         s.spawn(move || r.lookup(path));
//!     }
//! });
//! # Ok(())
//! # }
//! ```
//!
//! ## 動画キャッシュ
//!
//! ```rust,no_run
//! use ai_cache::{VideoCacheWriter, VideoCacheConfig, UpsertVideoRequest};
//! use file_feature_cache::{CacheConfig, DbLocation};
//!
//! # fn main() -> anyhow::Result<()> {
//! let writer = VideoCacheWriter::as_session(VideoCacheConfig {
//!     cache_config: CacheConfig {
//!         db_location:   DbLocation::AppCache(None),
//!         read_conns:    2,
//!         thumbnail_dir: Some("/var/cache/myapp/thumbs".into()),
//!     },
//!     thumbnail:   true,
//!     ffmpeg_path: Some("/usr/bin/ffmpeg".into()),
//! })?;
//!
//! writer.upsert(UpsertVideoRequest {
//!     file_path:       "/data/movie.mp4".to_string(),
//!     clip_vector:     Some(vec![0.1, 0.2]),
//!     wav2vec2_vector: Some(vec![0.3, 0.4]),
//! })?;
//! # Ok(())
//! # }
//! ```

mod core;
pub mod types;

pub use core::extension::MediaExtension;
pub use core::image::{ImageCacheConfig, ImageCacheReader, ImageCacheWriter};
pub use core::video::{VideoCacheConfig, VideoCacheReader, VideoCacheWriter};
pub use file_feature_cache::{CacheConfig, CacheError, DbLocation, Result};
pub use types::{
    ImageCacheEntry, ImageFeatures, LookupResult, UpsertImageRequest, UpsertVideoRequest,
    VideoCacheEntry, VideoFeatures,
};
