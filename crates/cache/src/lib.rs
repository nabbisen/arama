//! # ai_cache
//!
//! AI 推論アプリ向け SQLite キャッシュライブラリ。
//!
//! ファイル同一性確認 (ハッシュ計算・mtime チェック) はキャッシュストアが内部で担う。
//! アプリ側は **ファイルパスだけ** を渡せばよい。
//!
//! ## 基本的な使い方
//!
//! ```rust,no_run
//! use ai_cache::{CacheStore, UpsertImageRequest, LookupResult};
//!
//! # fn main() -> ai_cache::Result<()> {
//! // デフォルト設定で開く (SizeAdaptive: 4MB 以上は部分ハッシュ + mtime)
//! let store = CacheStore::open("cache.db", Default::default())?;
//!
//! // 登録 (hash / mtime はストアが自動計算)
//! store.upsert_image(UpsertImageRequest {
//!     file_path:      "/data/photo.jpg".to_string(),
//!     thumbnail_path: Some("/cache/thumb_photo.jpg".to_string()),
//!     clip_vector:    Some(vec![0.1, 0.2, 0.3]),
//! })?;
//!
//! // 照会 (ファイルが変更されていれば自動で Invalidated)
//! match store.lookup_image("/data/photo.jpg")? {
//!     LookupResult::Hit(entry) => {
//!         println!("thumbnail: {:?}", entry.thumbnail_path);
//!         println!("clip dims: {}", entry.features.unwrap().clip_vector.len());
//!     }
//!     LookupResult::Invalidated => println!("file changed, cache cleared"),
//!     LookupResult::Miss        => println!("not cached"),
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## ハッシュ戦略のカスタマイズ
//!
//! ```rust,no_run
//! use ai_cache::{CacheStore, store::CacheStoreConfig, identity::HashStrategy};
//!
//! # fn main() -> ai_cache::Result<()> {
//! let store = CacheStore::open("cache.db", CacheStoreConfig {
//!     read_conns:    Some(8),
//!     hash_strategy: HashStrategy::SizeAdaptive {
//!         threshold_bytes: 1 * 1024 * 1024, // 1 MB から部分ハッシュ
//!         partial_bytes:   128 * 1024,       // 128 KB × 2
//!     },
//! })?;
//! # Ok(())
//! # }
//! ```

pub mod error;
pub mod identity;
mod schema;
pub mod store;
pub mod types;

// よく使う型を re-export
pub use error::{Result, cache_error::CacheError};
pub use store::{cache_store::CacheStore, cashe_store_config::CacheStoreConfig};
pub use types::{
    ImageCacheEntry, ImageFeatures, LookupResult, UpsertImageRequest, UpsertVideoRequest,
    VideoCacheEntry, VideoFeatures,
};
