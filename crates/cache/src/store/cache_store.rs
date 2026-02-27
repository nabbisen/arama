use std::path::Path;

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::OpenFlags;

use super::cashe_store_config::CacheStoreConfig;
use crate::{CacheError, error::Result, schema::initialize, store::util::num_cpus};

// ---------------------------------------------------------------------------
// 型エイリアス
// ---------------------------------------------------------------------------

pub type ReadPool = Pool<SqliteConnectionManager>;
pub type WritePool = Pool<SqliteConnectionManager>;
pub type WriteConn = r2d2::PooledConnection<SqliteConnectionManager>;

// ---------------------------------------------------------------------------
// CacheStore
// ---------------------------------------------------------------------------

/// 読み書き分離コネクションプールを持つキャッシュストア。
///
/// - **read_pool**: WAL の読み取りスナップショット分離を活かし、rayon スレッド数分の
///   コネクションを同時保持する。読み取りは互いにブロックしない。
/// - **write_pool**: `max_size = 1` で直列化し、`busy_timeout` で一時競合を吸収する。
///
/// `Clone` はプール内部の `Arc` をコピーするだけで低コスト。
/// rayon の各タスクに `store.clone()` を渡す運用を想定している。
#[derive(Clone)]
pub struct CacheStore {
    pub read_pool: ReadPool,
    pub write_pool: WritePool,
    pub config: CacheStoreConfig,
}

impl CacheStore {
    /// 指定パスの SQLite ファイルを開く (存在しない場合は新規作成)。
    pub fn open(path: impl AsRef<Path>, config: CacheStoreConfig) -> Result<Self> {
        let path = path.as_ref();

        // --- 書き込みプール (1 コネクション) ---
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
            initialize(&conn)?;
        }

        // --- 読み取りプール (N コネクション) ---
        let n = config
            .read_conns
            .unwrap_or_else(|| num_cpus() as u32)
            .max(1);

        let read_manager = SqliteConnectionManager::file(path)
            .with_flags(OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_URI)
            .with_init(|c| {
                c.execute_batch(
                    "PRAGMA journal_mode = WAL;
                     PRAGMA foreign_keys = ON;",
                )
            });

        let read_pool = Pool::builder()
            .max_size(n)
            .build(read_manager)
            .map_err(CacheError::from_pool)?;

        Ok(Self {
            read_pool,
            write_pool,
            config,
        })
    }

    /// インメモリ DB を開く (テスト用)。
    pub fn open_in_memory() -> Result<Self> {
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
            initialize(&conn)?;
        }

        Ok(Self {
            read_pool: pool.clone(),
            write_pool: pool,
            config: CacheStoreConfig::default(),
        })
    }

    pub(crate) fn read(&self) -> Result<r2d2::PooledConnection<SqliteConnectionManager>> {
        self.read_pool.get().map_err(CacheError::from_pool)
    }

    pub(crate) fn write(&self) -> Result<WriteConn> {
        self.write_pool.get().map_err(CacheError::from_pool)
    }
}
