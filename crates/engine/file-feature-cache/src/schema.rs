//! SQLite スキーマの初期化とマイグレーション。

use rusqlite::Connection;

use crate::core::extension::CacheExtension;

const SCHEMA_VERSION: u32 = 1;

pub(crate) fn initialize<E: CacheExtension>(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA foreign_keys = ON;",
    )?;

    let current: u32 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;

    if current < SCHEMA_VERSION {
        migrate_v1(conn)?;
        E::migrate(conn)?;
        conn.execute_batch(&format!("PRAGMA user_version = {SCHEMA_VERSION};"))?;
    }

    Ok(())
}

fn migrate_v1(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS files (
            id         INTEGER PRIMARY KEY AUTOINCREMENT,
            path       TEXT    NOT NULL UNIQUE,   -- canonicalize 済み絶対パス
            file_hash  TEXT    NOT NULL,
            mtime_ns   INTEGER,
            created_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
            updated_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
        );
        CREATE INDEX IF NOT EXISTS idx_files_path ON files(path);
        ",
    )
}
