use std::path::Path;

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::OpenFlags;

use crate::CacheError;
use crate::config::CacheConfig;
use crate::core::schema::initialize;
use crate::core::store::util::num_cpus;
use crate::error::Result;
use crate::types::{ReadConn, WriteConn};

pub(crate) struct CacheStore {
    read_pool: Pool<SqliteConnectionManager>,
    write_pool: Pool<SqliteConnectionManager>,
    pub config: CacheConfig,
}

impl CacheStore {
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
            initialize(&conn)?;
        }

        let n = config.read_conns.unwrap_or_else(num_cpus).max(1);

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
            initialize(&conn)?;
        }

        Ok(Self {
            read_pool: pool.clone(),
            write_pool: pool,
            config,
        })
    }

    pub fn read(&self) -> Result<ReadConn> {
        self.read_pool.get().map_err(CacheError::from_pool)
    }

    pub fn write(&self) -> Result<WriteConn> {
        self.write_pool.get().map_err(CacheError::from_pool)
    }
}
