//! # ai_cache
//!
//! AI 推論アプリ向け SQLite キャッシュライブラリ。
//!
//! ## 権限モデル
//!
//! | 型 | 公開 API | 生成方法 |
//! |---|---|---|
//! | [`CacheWriter`] | lookup + upsert + delete + verify | `CacheWriter::open` |
//! | [`CacheReader`] | lookup のみ | `CacheWriter::as_reader()` |
//!
//! `CacheReader` は `CacheWriter` から生成する一方向の関係。逆方向への昇格はできない。
//! lookup 中に変更が検出された場合の内部 DELETE は `CacheReader` でも許容する。
//!
//! ## 使い方
//!
//! ```rust,no_run
//! use ai_cache::{CacheWriter, CacheReader, UpsertImageRequest, LookupResult};
//! use rayon::prelude::*;
//!
//! # fn main() -> ai_cache::Result<()> {
//! let writer = CacheWriter::open("cache.db", Default::default())?;
//! let reader = writer.as_reader(); // lookup のみの権限に落として配布
//!
//! // 更新権限が必要な箇所には writer を渡す
//! writer.upsert_image(UpsertImageRequest {
//!     file_path:      "/data/photo.jpg".to_string(),
//!     thumbnail_path: Some("/cache/thumb.jpg".to_string()),
//!     clip_vector:    Some(vec![0.1, 0.2, 0.3]),
//! })?;
//!
//! // 参照のみの箇所には reader を clone して配布
//! let paths = vec!["/data/photo.jpg", "/data/video.mp4"];
//! let results: Vec<_> = paths.par_iter()
//!     .map(|p| reader.clone().lookup_image(p))
//!     .collect();
//! # Ok(())
//! # }
//! ```
//!
//! ## ハッシュ戦略のカスタマイズ
//!
//! ```rust,no_run
//! use ai_cache::{CacheWriter, CacheConfig, identity::HashStrategy};
//!
//! # fn main() -> ai_cache::Result<()> {
//! let writer = CacheWriter::open("cache.db", CacheConfig {
//!     read_conns:    Some(8),
//!     hash_strategy: HashStrategy::SizeAdaptive {
//!         threshold_bytes: 1 * 1024 * 1024,
//!         partial_bytes:   128 * 1024,
//!     },
//! })?;
//! # Ok(())
//! # }
//! ```

mod schema;
pub(crate) mod store;

pub mod config;
pub mod error;
pub mod identity;
pub mod reader;
pub mod types;
pub mod writer;

// re-export
pub use error::{Result, cache_error::CacheError};
pub use identity::hash::hash_strategy::HashStrategy;
pub use reader::cache_reader::CacheReader;
pub use types::{
    ImageCacheEntry, ImageFeatures, LookupResult, UpsertImageRequest, UpsertVideoRequest,
    VideoCacheEntry, VideoFeatures,
};
pub use writer::{CacheConfig, cache_writer::CacheWriter};
