//! `CacheReader<E>` — 参照専用ハンドル。

use std::path::Path;
use std::sync::Arc;

use crate::CacheRead;
use crate::core::extension::CacheExtension;
use crate::core::store::{
    CacheConfig, CacheStore, DbLocation, db_delete_by_id, db_fetch_file_row, file_matches,
};
use crate::error::Result;

/// 参照専用ハンドル。
///
/// 読み取りしか必要としない場合に `CacheWriter<E>` を経由せず直接生成できる。
/// `Clone` コストは `Arc` のカウントアップのみ。rayon の各タスクに自由に配布できる。
#[derive(Clone)]
pub struct CacheReader<E: CacheExtension> {
    pub store: Arc<CacheStore<E>>,
}

impl<E: CacheExtension> CacheReader<E> {
    pub fn new(store: Arc<CacheStore<E>>) -> Self {
        Self { store }
    }

    /// 単発・使い捨て用。DB を開いて操作して閉じる想定。
    pub fn oneshot(location: DbLocation) -> Result<Self> {
        let config = CacheConfig {
            db_location: location,
            ..Default::default()
        };
        let path = config.db_location.resolve();
        let store = Arc::new(CacheStore::<E>::open(&path, config)?);
        Ok(Self { store })
    }

    /// 継続使用・rayon 並列処理用セッション。
    pub fn as_session(config: CacheConfig) -> Result<Self> {
        let path = config.db_location.resolve();
        let store = Arc::new(CacheStore::<E>::open(&path, config)?);
        Ok(Self { store })
    }

    /// インメモリ DB を開く (テスト用)。
    pub fn open_in_memory() -> Result<Self> {
        let store = Arc::new(CacheStore::<E>::open_in_memory(CacheConfig::default())?);
        Ok(Self { store })
    }

    /// 登録済みファイルパスの一覧を返す。
    pub fn list_paths(&self) -> Result<Vec<String>> {
        let conn = self.store.read()?;
        let mut stmt = conn.prepare("SELECT file_path FROM files ORDER BY file_path")?;
        let paths = stmt
            .query_map([], |r| r.get::<_, String>(0))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(paths)
    }

    /// ファイルが DB に存在し、かつ現在のファイルと一致するか確認する。
    ///
    /// 変更が検出された場合は古いレコードを内部で削除し `false` を返す。
    pub fn check(&self, file_path: &str) -> Result<bool> {
        let path = Path::new(file_path);
        match db_fetch_file_row(&self.store, file_path)? {
            None => Ok(false),
            Some((file_id, stored_hash, stored_mtime)) => {
                if file_matches(&self.store, &stored_hash, stored_mtime, path)? {
                    Ok(true)
                } else {
                    db_delete_by_id(&self.store, file_id)?;
                    Ok(false)
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// CacheRead trait 実装
// ---------------------------------------------------------------------------

impl<E: CacheExtension> CacheRead for CacheReader<E> {
    fn check(&self, file_path: &str) -> Result<bool> {
        CacheReader::check(self, file_path)
    }

    fn list_paths(&self) -> Result<Vec<String>> {
        CacheReader::list_paths(self)
    }
}
