use rusqlite::Connection;

use crate::core::extension::CacheExtension;

/// 最新スキーマバージョン
const SCHEMA_VERSION: u32 = 1;

/// `files` テーブルを初期化し、`CacheExtension::migrate` で拡張テーブルを作成する。
pub fn initialize<E: CacheExtension>(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch("PRAGMA journal_mode = WAL;")?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;

    let current_version: u32 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;

    if current_version < SCHEMA_VERSION {
        migrate_v1(conn)?;
        E::migrate(conn)?;
        conn.execute_batch(&format!("PRAGMA user_version = {SCHEMA_VERSION};"))?;
    }

    Ok(())
}

/// `files` テーブル (v0 → v1)
fn migrate_v1(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS files (
            id         INTEGER PRIMARY KEY AUTOINCREMENT,
            file_path  TEXT    NOT NULL UNIQUE,
            file_hash  TEXT    NOT NULL,
            mtime_ns   INTEGER,
            created_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
            updated_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
        );
        CREATE INDEX IF NOT EXISTS idx_files_path ON files(file_path);
    ",
    )
}
