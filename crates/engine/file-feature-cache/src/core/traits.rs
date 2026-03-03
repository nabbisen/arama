//! `CacheWrite` / `CacheRead` — キャッシュハンドルの共通インターフェース。
//!
//! 特化クレートはこれらの trait を実装することで、
//! エンジンとの統一されたインターフェースを提供する。

use crate::core::store::DbLocation;
use crate::error::Result;

/// 参照専用ハンドルの共通インターフェース。
pub trait CacheRead {
    /// ファイルが DB に存在し、かつ現在のファイルと一致するか確認する。
    ///
    /// 変更が検出された場合は古いレコードを内部で削除し `false` を返す。
    fn check(&self, file_path: &str) -> Result<bool>;

    /// 登録済みファイルパスの一覧を返す。
    fn list_paths(&self) -> Result<Vec<String>>;
}

/// 参照 + 更新ハンドルの共通インターフェース。
pub trait CacheWrite: Sized {
    /// 対応する Reader の型。
    type Reader: CacheRead;

    /// 対応する Config の型。
    type Config;

    /// 継続使用・rayon 並列処理用セッションを開く。
    fn as_session(config: Self::Config) -> Result<Self>;

    /// 単発・使い捨て用。その他の設定はデフォルト値を使う。
    fn oneshot(location: DbLocation) -> Result<Self>;

    /// 参照専用の Reader にダウングレードする。
    ///
    /// 内部の `Arc<CacheStore>` を共有するため追加コストなし。
    fn as_reader(&self) -> Self::Reader;

    /// ファイルパスに紐付くキャッシュを全て削除する。
    /// 戻り値: 対象レコードが存在した場合 `true`
    fn delete(&self, file_path: &str) -> Result<bool>;

    /// ファイルの現在の状態を確認し、変更されていれば DB から削除して `false` を返す。
    fn verify_or_invalidate(&self, file_path: &str) -> Result<bool>;

    /// 登録済みファイルパスの一覧を返す。
    fn list_paths(&self) -> Result<Vec<String>>;
}
