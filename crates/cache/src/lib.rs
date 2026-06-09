//! # arama-cache
//!
//! Caches AI inference results (thumbnails + feature vectors) for image
//! and video files in SQLite, backed by the [`localcache`] engine
//! (RFC 002). The cache database holds two namespaces — `image` and
//! `video` — in a single file, with per-file freshness tracked by
//! metadata-first change detection.
//!
//! ## Choosing a handle
//!
//! | Type | Purpose |
//! |---|---|
//! | [`ImageCacheWriter`] | Register, look up, and delete image entries |
//! | [`ImageCacheReader`] | Look up image entries (parallel-friendly) |
//! | [`VideoCacheWriter`] | Register, look up, and delete video entries |
//! | [`VideoCacheReader`] | Look up video entries (parallel-friendly) |
//!
//! Writers serialize database writes through a single connection; both
//! writers and readers serve lookups from a pool of `read_conns`
//! read-only connections, so cloned readers can fan lookups out across
//! threads.
//!
//! ## Basic usage
//!
//! ```rust,no_run
//! use arama_cache::{
//!     CacheConfig, DbLocation, ImageCacheConfig, ImageCacheWriter, LookupResult,
//!     UpsertImageRequest,
//! };
//!
//! # fn main() -> anyhow::Result<()> {
//! let writer = ImageCacheWriter::as_session(ImageCacheConfig {
//!     cache_config: CacheConfig {
//!         db_location: DbLocation::AppCache(None),
//!         read_conns: 4,
//!         thumbnail_dir: Some("/var/cache/myapp/thumbs".into()),
//!     },
//! })?;
//!
//! writer.upsert(UpsertImageRequest {
//!     path: "/data/photo.jpg".into(),
//!     clip_vector: Some(vec![0.1, 0.2, 0.3]),
//! })?;
//!
//! match writer.lookup(std::path::Path::new("/data/photo.jpg"))? {
//!     LookupResult::Hit(entry) => {
//!         println!("thumbnail: {:?}", entry.thumbnail_path);
//!         println!("features:  {:?}", entry.features);
//!     }
//!     LookupResult::Invalidated => println!("file changed; will be recomputed"),
//!     LookupResult::Miss => println!("not cached"),
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## `onetime` — single-shot calls
//!
//! ```rust,no_run
//! use arama_cache::{DbLocation, ImageCacheWriter};
//!
//! # fn main() -> anyhow::Result<()> {
//! let result = ImageCacheWriter::onetime(DbLocation::WorkDir(None))?
//!     .lookup(std::path::Path::new("/data/photo.jpg"))?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Migrating from the v1 cache
//!
//! Applications upgrading from the `file-feature-cache`-backed v1
//! database run [`migrate_v1_if_present`] once at startup; it is a no-op
//! when there is nothing to migrate.

mod core;
pub mod types;

pub use core::engine::{CacheConfig, CacheError, DbLocation, Result};
pub use core::image::{ImageCacheConfig, ImageCacheReader, ImageCacheWriter};
pub use core::migrate::{MigrationReport, migrate_v1_if_present};
pub use core::video::{VideoCacheConfig, VideoCacheReader, VideoCacheWriter};
pub use types::{
    CacheRead, ImageCacheEntry, ImageFeatures, LookupResult, UpsertImageRequest,
    UpsertVideoRequest, VideoCacheEntry, VideoFeatures,
};
