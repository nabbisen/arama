//! [`CacheExtension`] — 特化クレートがスキーマ拡張を注入するためのトレイト。

use rusqlite::Connection;

/// 特化クレートが実装するトレイト。
///
/// エンジンが `files` テーブルを作成した直後に [`migrate`] が呼ばれる。
/// 拡張テーブル (`image_features` 等) はここで `CREATE TABLE IF NOT EXISTS` する。
///
/// # 例
///
/// ```rust
/// use file_feature_cache::CacheExtension;
/// use rusqlite::Connection;
///
/// #[derive(Clone)]
/// pub struct ScoreExtension;
///
/// impl CacheExtension for ScoreExtension {
///     fn migrate(conn: &Connection) -> rusqlite::Result<()> {
///         conn.execute_batch("
///             CREATE TABLE IF NOT EXISTS scores (
///                 file_id INTEGER PRIMARY KEY REFERENCES files(id) ON DELETE CASCADE,
///                 score   REAL NOT NULL
///             );
///         ")
///     }
/// }
/// ```
///
/// [`migrate`]: CacheExtension::migrate
pub trait CacheExtension: Send + Sync + Clone + 'static {
    /// 拡張テーブルを作成する。
    ///
    /// `files` テーブルは作成済みの状態で呼ばれる。
    /// 外部キー `REFERENCES files(id) ON DELETE CASCADE` を使って
    /// `files` 削除時に自動的に拡張レコードも削除されるよう定義すること。
    fn migrate(conn: &Connection) -> rusqlite::Result<()>;
}

/// 拡張なし。エンジン単体のテストや `files` テーブルのみで十分な場合に使う。
#[derive(Clone)]
pub struct NoExtension;

impl CacheExtension for NoExtension {
    fn migrate(_conn: &Connection) -> rusqlite::Result<()> {
        Ok(())
    }
}
