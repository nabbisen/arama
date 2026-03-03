//! # file_feature_cache
//!
//! ファイルに紐付く特徴量を SQLite にキャッシュする汎用エンジン。
//!
//! このクレート自体はスキーマ拡張を提供しない。
//! [`CacheExtension`] を実装した特化クレートが拡張テーブルを定義する。
//!
//! ## 使い方 (特化クレート側)
//!
//! ```rust,no_run
//! use file_feature_cache::{CacheExtension, CacheWriter, DbLocation};
//! use rusqlite::Connection;
//!
//! pub struct MyExtension;
//!
//! impl CacheExtension for MyExtension {
//!     fn migrate(conn: &Connection) -> rusqlite::Result<()> {
//!         conn.execute_batch("
//!             CREATE TABLE IF NOT EXISTS my_features (
//!                 file_id INTEGER PRIMARY KEY REFERENCES files(id) ON DELETE CASCADE,
//!                 vector  BLOB NOT NULL
//!             );
//!         ")
//!     }
//! }
//!
//! # fn main() -> file_feature_cache::Result<()> {
//! type MyWriter = CacheWriter<MyExtension>;
//!
//! let writer = MyWriter::open(DbLocation::WorkDir(None))?;
//! let file_id = writer.upsert_file("/data/doc.pdf")?;
//! // → file_id を使って my_features テーブルに書き込む
//! # Ok(())
//! # }
//! ```

pub mod config;
mod core;
pub mod error;

pub use core::extension::{CacheExtension, NoExtension};
pub use core::reader::CacheReader;
pub use core::store::{CacheConfig, DbLocation};
pub use core::traits::{CacheRead, CacheWrite};
pub use core::writer::CacheWriter;
pub use error::{CacheError, Result};
