//! `CacheStore` — コネクションプール・設定・DB ヘルパー関数を一元管理するコア。

use std::marker::PhantomData;
use std::path::{Path, PathBuf};

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{OpenFlags, OptionalExtension};

use crate::core::extension::CacheExtension;
use crate::core::identity::{FileFingerprint, HashStrategy, compute, matches_stored};
use crate::core::schema::initialize;
use crate::error::{CacheError, Result};

// ---------------------------------------------------------------------------
// DbLocation
// ---------------------------------------------------------------------------

/// DB ファイルの場所を表す。
#[derive(Debug, Clone)]
pub enum DbLocation {
    /// パスを完全に指定する。
    Custom(PathBuf),

    /// XDG キャッシュディレクトリを使う。
    ///
    /// アプリ名は `std::env::current_exe()` で実行時に自動取得する。
    /// `file_name` が `None` の場合は `cache.db`。
    AppCache(Option<String>),

    /// 実行ディレクトリに作成する (デフォルト)。
    ///
    /// `file_name` が `None` の場合は `cache.db`。
    WorkDir(Option<String>),
}

impl Default for DbLocation {
    fn default() -> Self {
        Self::WorkDir(None)
    }
}

impl DbLocation {
    pub(crate) fn resolve(&self) -> PathBuf {
        match self {
            Self::Custom(p) => p.clone(),

            Self::AppCache(file_name) => {
                let base = std::env::var("XDG_CACHE_HOME")
                    .map(PathBuf::from)
                    .unwrap_or_else(|_| {
                        std::env::var("HOME")
                            .map(|h| PathBuf::from(h).join(".cache"))
                            .unwrap_or_else(|_| PathBuf::from(".cache"))
                    });
                let app = std::env::current_exe()
                    .ok()
                    .and_then(|p| p.file_stem().map(|s| s.to_string_lossy().into_owned()))
                    .unwrap_or_else(|| "app".to_string());
                let name = file_name.as_deref().unwrap_or("cache.db");
                base.join(app).join(name)
            }

            Self::WorkDir(file_name) => {
                let name = file_name.as_deref().unwrap_or("cache.db");
                PathBuf::from(format!("./{name}"))
            }
        }
    }
}

// ---------------------------------------------------------------------------
// CacheConfig
// ---------------------------------------------------------------------------

/// [`CacheWriter::as_session`] に渡す設定。
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// DB ファイルの場所。
    pub db_location: DbLocation,
    /// 読み取りプールのコネクション数。
    pub read_conns: u32,
    /// サムネイルファイルを格納するディレクトリ。
    /// `None` の場合はサムネイル管理を行わない。
    pub thumbnail_dir: Option<PathBuf>,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            db_location: DbLocation::default(),
            read_conns: num_cpus(),
            thumbnail_dir: None,
        }
    }
}

// ---------------------------------------------------------------------------
// 型エイリアス
// ---------------------------------------------------------------------------

pub(crate) type ReadConn = r2d2::PooledConnection<SqliteConnectionManager>;
pub(crate) type WriteConn = r2d2::PooledConnection<SqliteConnectionManager>;

// ---------------------------------------------------------------------------
// CacheStore
// ---------------------------------------------------------------------------

pub struct CacheStore<E: CacheExtension> {
    read_pool: Pool<SqliteConnectionManager>,
    write_pool: Pool<SqliteConnectionManager>,
    pub config: CacheConfig,
    _ext: PhantomData<E>,
}

impl<E: CacheExtension> CacheStore<E> {
    pub fn open(path: &Path, config: CacheConfig) -> Result<Self> {
        let write_manager = SqliteConnectionManager::file(path)
            .with_flags(
                OpenFlags::SQLITE_OPEN_READ_WRITE
                    | OpenFlags::SQLITE_OPEN_CREATE
                    | OpenFlags::SQLITE_OPEN_URI,
            )
            .with_init(|c| {
                c.execute_batch(
                    "PRAGMA journal_mode = WAL;
                 PRAGMA foreign_keys = ON;
                 PRAGMA synchronous   = NORMAL;
                 PRAGMA busy_timeout  = 5000;",
                )
            });

        let write_pool = Pool::builder()
            .max_size(1)
            .build(write_manager)
            .map_err(CacheError::from_pool)?;

        {
            let conn = write_pool.get().map_err(CacheError::from_pool)?;
            initialize::<E>(&conn)?;
        }

        let read_manager = SqliteConnectionManager::file(path)
            .with_flags(OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_URI)
            .with_init(|c| {
                c.execute_batch(
                    "PRAGMA journal_mode = WAL;
                 PRAGMA foreign_keys = ON;",
                )
            });

        let read_pool = Pool::builder()
            .max_size(config.read_conns.max(1))
            .build(read_manager)
            .map_err(CacheError::from_pool)?;

        Ok(Self {
            read_pool,
            write_pool,
            config,
            _ext: PhantomData,
        })
    }

    pub fn open_in_memory(config: CacheConfig) -> Result<Self> {
        let manager = SqliteConnectionManager::memory().with_init(|c| {
            c.execute_batch(
                "PRAGMA journal_mode = WAL;
                 PRAGMA foreign_keys = ON;",
            )
        });

        let pool = Pool::builder()
            .max_size(1)
            .build(manager)
            .map_err(CacheError::from_pool)?;

        {
            let conn = pool.get().map_err(CacheError::from_pool)?;
            initialize::<E>(&conn)?;
        }

        Ok(Self {
            read_pool: pool.clone(),
            write_pool: pool,
            config,
            _ext: PhantomData,
        })
    }

    pub fn read(&self) -> Result<ReadConn> {
        self.read_pool.get().map_err(CacheError::from_pool)
    }

    pub fn write(&self) -> Result<WriteConn> {
        self.write_pool.get().map_err(CacheError::from_pool)
    }
}

// ---------------------------------------------------------------------------
// DB ヘルパー関数 — files テーブル
// ---------------------------------------------------------------------------

pub(crate) fn db_fetch_file_row<E: CacheExtension>(
    store: &CacheStore<E>,
    file_path: &str,
) -> Result<Option<(i64, String, Option<i64>)>> {
    let conn = store.read()?;
    let row = conn
        .query_row(
            "SELECT id, file_hash, mtime_ns FROM files WHERE file_path = ?1",
            [file_path],
            |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, Option<i64>>(2)?,
                ))
            },
        )
        .optional()?;
    Ok(row)
}

pub(crate) fn db_upsert_file(
    conn: &WriteConn,
    file_path: &str,
    hash: &str,
    mtime_ns: Option<i64>,
) -> rusqlite::Result<i64> {
    conn.execute(
        "INSERT INTO files (file_path, file_hash, mtime_ns, updated_at)
         VALUES (?1, ?2, ?3, strftime('%s','now'))
         ON CONFLICT(file_path) DO UPDATE
             SET file_hash  = excluded.file_hash,
                 mtime_ns   = excluded.mtime_ns,
                 updated_at = strftime('%s','now')",
        rusqlite::params![file_path, hash, mtime_ns],
    )?;
    conn.query_row(
        "SELECT id FROM files WHERE file_path = ?1",
        [file_path],
        |r| r.get::<_, i64>(0),
    )
}

pub(crate) fn db_delete_by_id<E: CacheExtension>(
    store: &CacheStore<E>,
    file_id: i64,
) -> Result<()> {
    let conn = store.write()?;
    conn.execute("DELETE FROM files WHERE id = ?1", [file_id])?;
    Ok(())
}

// ---------------------------------------------------------------------------
// 共有ユーティリティ
// ---------------------------------------------------------------------------

pub(crate) fn file_matches<E: CacheExtension>(
    store: &CacheStore<E>,
    stored_hash: &str,
    stored_mtime: Option<i64>,
    path: &Path,
) -> Result<bool> {
    matches_stored(
        stored_hash,
        stored_mtime,
        path,
        &store.config.hash_strategy_internal(),
    )
    .map_err(|e| CacheError::io(path, e))
}

pub(crate) fn compute_fingerprint<E: CacheExtension>(
    store: &CacheStore<E>,
    path: &Path,
) -> Result<FileFingerprint> {
    compute(path, &store.config.hash_strategy_internal()).map_err(|e| CacheError::io(path, e))
}

// ---------------------------------------------------------------------------
// 内部ユーティリティ
// ---------------------------------------------------------------------------

fn num_cpus() -> u32 {
    std::thread::available_parallelism()
        .map(|n| n.get() as u32)
        .unwrap_or(4)
}

impl CacheConfig {
    /// 内部で使用する HashStrategy (外部非公開)
    pub(crate) fn hash_strategy_internal(&self) -> HashStrategy {
        HashStrategy::default()
    }
}

// ---------------------------------------------------------------------------
// ユニットテスト
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn db_location_custom_resolves_to_given_path() {
        let loc = DbLocation::Custom("/tmp/myapp/cache.db".into());
        assert_eq!(loc.resolve(), PathBuf::from("/tmp/myapp/cache.db"));
    }

    #[test]
    fn db_location_workdir_default_filename() {
        let loc = DbLocation::WorkDir(None);
        assert_eq!(loc.resolve(), PathBuf::from("./cache.db"));
    }

    #[test]
    fn db_location_workdir_custom_filename() {
        let loc = DbLocation::WorkDir(Some("inference.db".into()));
        assert_eq!(loc.resolve(), PathBuf::from("./inference.db"));
    }

    #[test]
    fn db_location_appcache_default_filename() {
        let loc = DbLocation::AppCache(None);
        let s = loc.resolve().to_str().unwrap().to_string();
        assert!(s.ends_with("cache.db"), "unexpected: {s}");
    }
}
