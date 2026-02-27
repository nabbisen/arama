use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

use crate::{CacheError, Result, schema::initialize};

use super::{
    cache_store::{CacheStore, WriteConn},
    cashe_store_config::CacheStoreConfig,
};

// ---------------------------------------------------------------------------
// 内部ユーティリティ
// ---------------------------------------------------------------------------

pub fn upsert_file_record(
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

pub fn num_cpus() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
}

// テスト用: config 付きインメモリ DB
impl CacheStore {
    #[doc(hidden)]
    pub fn open_in_memory_with_config(config: CacheStoreConfig) -> Result<Self> {
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
}
