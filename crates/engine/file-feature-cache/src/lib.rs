mod core;
mod error;
mod schema;

pub use core::extension::{CacheExtension, NoExtension};
pub use core::reader::CacheReader;
pub use core::store::{CacheConfig, DbLocation};
pub use core::writer::CacheWriter;
pub use error::{CacheError, Result};

use std::path::{Path, PathBuf};

/// 参照専用ハンドルの共通インターフェース。
pub trait CacheRead {
    /// ファイルが DB に登録済みで、かつ現在の内容と一致するか確認する。
    ///
    /// ファイルが存在しない、または内容が変更されていた場合は
    /// 古いレコードを削除して `false` を返す。
    fn check(&self, path: &Path) -> Result<bool>;

    /// `check` の一括版。各パスに対して `(PathBuf, Result<bool>)` を返す。
    ///
    /// read pool の複数コネクションを利用して rayon で並列実行される。
    /// 個々のエラーは `Err` として各要素に格納され、他のパスの処理は継続する。
    fn check_all(&self, paths: &[&Path]) -> Vec<(PathBuf, Result<bool>)>;

    /// 登録済みパスの一覧を返す (アルファベット順)。
    ///
    /// DB に保存されているのは正規化済みの絶対パスである。
    fn list_paths(&self) -> Result<Vec<String>>;
}

/// 参照 + 更新ハンドルの共通インターフェース。
pub trait CacheWrite: Sized {
    type Reader: CacheRead;

    fn as_session(config: CacheConfig) -> Result<Self>;
    fn onetime(location: DbLocation) -> Result<Self>;
    fn as_reader(&self) -> Self::Reader;

    /// `files` テーブルが現在のファイル内容と一致することを保証し、`id` を返す。
    ///
    /// | 状態 | 動作 |
    /// |---|---|
    /// | 未登録 | fingerprint を計算して INSERT → 新しい `id` |
    /// | 登録済み・内容一致 | DB 書き込みなし → 既存の `id` (fast path) |
    /// | 登録済み・内容変更 | 旧レコードを DELETE して再 INSERT → 新しい `id` |
    ///
    /// ファイルが存在しない場合は `Err(CacheError::Io)` を返す。
    fn refresh(&self, path: &Path) -> Result<i64>;

    /// `refresh` の一括版。各パスに対して `(PathBuf, Result<i64>)` を返す。
    ///
    /// fingerprint 計算 (I/O + SHA-256) を rayon で並列実行し、
    /// DB 書き込みは write pool の制約に従い直列で行う。
    /// 個々のエラーは `Err` として各要素に格納され、他のパスの処理は継続する。
    fn refresh_all(&self, paths: &[&Path]) -> Vec<(PathBuf, Result<i64>)>;

    /// ファイルに紐付くキャッシュを全て削除する。
    ///
    /// `FOREIGN KEY ... ON DELETE CASCADE` により拡張テーブルも削除される。
    /// 戻り値: 対象レコードが存在した場合 `true`。
    /// ファイルが存在しない場合は `Err(CacheError::Io)` を返す。
    fn delete(&self, path: &Path) -> Result<bool>;

    fn list_paths(&self) -> Result<Vec<String>>;
}
