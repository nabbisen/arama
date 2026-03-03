//! [`CacheExtension`] trait — 拡張テーブルの定義をエンジンに注入する。

use rusqlite::Connection;

/// 特化 crate が実装するトレイト。
///
/// エンジンが `files` テーブルを作成した直後に [`migrate`] が呼ばれる。
/// 拡張テーブル (`image_features` 等) はここで作成する。
///
/// [`migrate`]: CacheExtension::migrate
pub trait CacheExtension: Send + Sync + Clone + 'static {
    /// 拡張テーブルを作成する。
    ///
    /// `files` テーブルは作成済みの状態で呼ばれる。
    /// `file_id` を外部キーとして参照するテーブルをここで定義する。
    fn migrate(conn: &Connection) -> rusqlite::Result<()>;
}

/// 拡張なし。エンジン単体のテストや、`files` テーブルのみで十分な場合に使う。
#[derive(Clone)]
pub struct NoExtension;

impl CacheExtension for NoExtension {
    fn migrate(_conn: &Connection) -> rusqlite::Result<()> {
        Ok(())
    }
}
