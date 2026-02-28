//! # arama_cache
//!
//! AI 推論アプリ向け SQLite キャッシュライブラリ。
//!
//! ## DB ファイルの場所
//!
//! [`DbLocation`] で DB の置き場所を指定する。デフォルトは `./arama_cache.db`。
//!
//! | バリアント | パス例 | 用途 |
//! |---|---|---|
//! | `DbLocation::Custom(path)` | 任意のパス | アプリが完全に制御したい場合 |
//! | `DbLocation::AppCache(..)` | `~/.cache/<実行バイナリ名>/cache.db` | XDG を明示的に使いたい場合 |
//! | `DbLocation::WorkDir(..)` (デフォルト) | `./arama_cache.db` | 手軽に始める場合 |
//!
//! ## API の選び方
//!
//! | 用途 | API |
//! |---|---|
//! | rayon 並列処理・繰り返し呼び出し | [`reader::session`] / [`writer::session`] (primary) |
//! | 単発処理・初期化・スクリプト | [`reader::oneshot`] / [`writer::oneshot`] |
//!
//! ## 基本的な使い方 — Session API
//!
//! ```rust,no_run
//! use arama_cache::{CacheWriter, UpsertImageRequest, LookupResult};
//!
//! # fn main() -> arama_cache::Result<()> {
//! let writer = CacheWriter::open()?;
//!
//! writer.upsert_image(UpsertImageRequest {
//!     file_path:      "/data/photo.jpg".to_string(),
//!     thumbnail_path: Some("/cache/thumb.jpg".to_string()),
//!     clip_vector:    Some(vec![0.1, 0.2, 0.3]),
//! })?;
//!
//! match writer.lookup_image("/data/photo.jpg")? {
//!     LookupResult::Hit(entry) => println!("clip dims: {}", entry.features.unwrap().clip_vector.len()),
//!     LookupResult::Invalidated => println!("file changed, cache cleared"),
//!     LookupResult::Miss        => println!("not cached"),
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## 権限モデル — CacheWriter と CacheReader
//!
//! `CacheWriter` から `as_reader()` で参照専用の [`CacheReader`] を生成できる。
//! どちらも `Clone` が低コストなので rayon の各タスクに自由に配布できる。
//!
//! ```rust,no_run
//! use arama_cache::{CacheWriter, CacheReader, LookupResult};
//! use rayon::prelude::*;
//!
//! # fn main() -> arama_cache::Result<()> {
//! let writer = CacheWriter::open()?;
//! let reader: CacheReader = writer.as_reader();
//!
//! let paths = vec!["/data/a.jpg", "/data/b.jpg", "/data/c.mp4"];
//! let results: Vec<_> = paths
//!     .par_iter()
//!     .map(|p| reader.clone().lookup_image(p))
//!     .collect();
//! # Ok(())
//! # }
//! ```
//!
//! ## 単発呼び出し — Oneshot API
//!
//! `&self` なしで呼べる free function。
//! 呼び出しのたびに DB を開き直すため、ループや並列処理には使わないこと。
//!
//! ```rust,no_run
//! use arama_cache::{reader, writer, UpsertImageRequest, LookupResult};
//!
//! # fn main() -> arama_cache::Result<()> {
//! writer::oneshot::upsert_image(UpsertImageRequest {
//!     file_path:      "/data/photo.jpg".to_string(),
//!     thumbnail_path: None,
//!     clip_vector:    Some(vec![0.1, 0.2, 0.3]),
//! })?;
//!
//! match reader::oneshot::lookup_image("/data/photo.jpg")? {
//!     LookupResult::Hit(entry) => println!("hit"),
//!     LookupResult::Invalidated | LookupResult::Miss => println!("no cache"),
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## カスタム設定
//!
//! ```rust,no_run
//! use arama_cache::{CacheWriter, CacheConfig, DbLocation};
//!
//! # fn main() -> arama_cache::Result<()> {
//! let writer = CacheWriter::open_with_config(CacheConfig {
//!     db_location: DbLocation::Custom("/var/myapp/cache.db".into()),
//!     ..Default::default()
//! })?;
//! # Ok(())
//! # }
//! ```

pub mod config;
pub(crate) mod core;
pub mod error;
pub mod types;

// re-export
pub use config::cache_config::CacheConfig;
pub use core::identity::hash::hash_strategy::HashStrategy;
pub use core::reader::{self, cache_reader::CacheReader};
pub use core::writer::{self, cache_writer::CacheWriter};
pub use error::{CacheError, Result};
pub use types::{
    ImageCacheEntry, ImageFeatures, LookupResult, UpsertImageRequest, UpsertVideoRequest,
    VideoCacheEntry, VideoFeatures,
};
