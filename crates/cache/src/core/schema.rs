use rusqlite::Connection;

/// 最新スキーマバージョン
const SCHEMA_VERSION: u32 = 1;

/// テーブルを初期化し、WAL モードを有効化する。
/// 既存 DB に対しては将来の migration フックを呼ぶ構造にしてある。
pub fn initialize(conn: &Connection) -> rusqlite::Result<()> {
    // WAL モードで書き込み競合を低減
    conn.execute_batch("PRAGMA journal_mode = WAL;")?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;

    let current_version: u32 =
        conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;

    if current_version < SCHEMA_VERSION {
        apply_migrations(conn, current_version)?;
        conn.execute_batch(&format!("PRAGMA user_version = {SCHEMA_VERSION};"))?;
    }

    Ok(())
}

fn apply_migrations(conn: &Connection, from: u32) -> rusqlite::Result<()> {
    if from < 1 {
        migrate_v1(conn)?;
    }
    // 将来: if from < 2 { migrate_v2(conn)?; } ...
    Ok(())
}

/// 初期スキーマ (v0 → v1)
fn migrate_v1(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch("
        -- ファイルメタ情報テーブル。file_path が一意キー。
        -- mtime_ns は Optional (HashOnly モード時は NULL)。
        CREATE TABLE IF NOT EXISTS files (
            id         INTEGER PRIMARY KEY AUTOINCREMENT,
            file_path  TEXT    NOT NULL UNIQUE,
            file_hash  TEXT    NOT NULL,
            mtime_ns   INTEGER,          -- UNIX epoch nanoseconds; NULL if not tracked
            created_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
            updated_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
        );

        -- サムネイル情報 (1 ファイル : 1 サムネイル)
        CREATE TABLE IF NOT EXISTS thumbnails (
            file_id        INTEGER PRIMARY KEY REFERENCES files(id) ON DELETE CASCADE,
            thumbnail_path TEXT NOT NULL
        );

        -- 画像ファイル用特徴量 (CLIP のみ)
        -- clip_vector: little-endian f32 配列を BLOB で保存
        CREATE TABLE IF NOT EXISTS image_features (
            file_id     INTEGER PRIMARY KEY REFERENCES files(id) ON DELETE CASCADE,
            clip_vector BLOB    NOT NULL
        );

        -- 動画ファイル用特徴量 (CLIP + wav2vec2)
        CREATE TABLE IF NOT EXISTS video_features (
            file_id          INTEGER PRIMARY KEY REFERENCES files(id) ON DELETE CASCADE,
            clip_vector      BLOB    NOT NULL,
            wav2vec2_vector  BLOB    NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_files_path ON files(file_path);
    ")
}
