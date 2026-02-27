//! # arama_cache
//!
//! AI 推論アプリ向け SQLite キャッシュライブラリ。
//!
//! ## DB ファイルの場所
//!
//! DB パスはライブラリが以下の優先順位で自動解決する。
//!
//! 1. [`CacheConfig::db_path`] が `Some` ならそのパス  ← アプリが明示指定する主経路
//! 2. 環境変数 `arama_cache_DB` が設定されていればそのパス
//! 3. `$XDG_CACHE_HOME/arama_cache/cache.db`
//!    (未設定時は `$HOME/.cache/arama_cache/cache.db`)
//! 4. カレントディレクトリの `./arama_cache.db`
//!
//! ## API の選び方
//!
//! | 用途 | API |
//! |---|---|
//! | rayon 並列処理・繰り返し呼び出し | [`CacheWriter`] / [`CacheReader`] (primary) |
//! | 単発スクリプト・初期化処理 | [`convenience`] モジュールの free function |
//!
//! ## 基本的な使い方 — Primary API
//!
//! ```rust,no_run
//! use arama_cache::{CacheWriter, UpsertImageRequest, LookupResult};
//!
//! # fn main() -> arama_cache::Result<()> {
//! // DB パスは自動解決。ほとんどの用途はこれで十分。
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
//!
//! // lookup しか必要ない箇所には権限を落とした CacheReader を配布する
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
//! ## Convenience API — 単発呼び出し
//!
//! `&self` なしで呼べる free function。
//! 呼び出しのたびに DB を開き直すため、ループや並列処理には使わないこと。
//!
//! ```rust,no_run
//! use arama_cache::{convenience, UpsertImageRequest, LookupResult};
//!
//! # fn main() -> arama_cache::Result<()> {
//! // 初期化処理やスクリプト的な単発処理に
//! convenience::upsert_image(UpsertImageRequest {
//!     file_path:      "/data/photo.jpg".to_string(),
//!     thumbnail_path: None,
//!     clip_vector:    Some(vec![0.1, 0.2, 0.3]),
//! })?;
//!
//! match convenience::lookup_image("/data/photo.jpg")? {
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
//! use arama_cache::{CacheWriter, CacheConfig, HashStrategy};
//!
//! # fn main() -> arama_cache::Result<()> {
//! // アプリ側でパスを明示指定する場合 (unsafe な set_var 不要)
//! let writer = CacheWriter::open_with_config(CacheConfig {
//!     db_path:       Some("/var/cache/myapp/cache.db".into()),
//!     read_conns:    None,
//!     hash_strategy: HashStrategy::default(),
//! })?;
//!
//! // ハッシュ戦略だけ変えてパスは自動解決に任せる場合
//! let writer = CacheWriter::open_with_config(CacheConfig {
//!     db_path:       None,
//!     read_conns:    Some(8),
//!     hash_strategy: HashStrategy::Full,
//! })?;
//! # Ok(())
//! # }
//! ```

pub mod config;
pub(crate) mod core;
pub mod error;
pub mod types;

// re-export
pub use config::CacheConfig;
pub use core::identity::hash::hash_strategy::HashStrategy;
pub use core::reader::{self, cache_reader::CacheReader};
pub use core::writer::{self, cache_writer::CacheWriter};
pub use error::{CacheError, Result};
pub use types::{
    ImageCacheEntry, ImageFeatures, LookupResult, UpsertImageRequest, UpsertVideoRequest,
    VideoCacheEntry, VideoFeatures,
};
