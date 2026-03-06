//! `CacheReader<E>` — 参照専用ハンドル。

use std::path::{Path, PathBuf};
use std::sync::Arc;

use rayon::prelude::*;

use crate::CacheRead;
use crate::core::extension::CacheExtension;
use crate::core::store::{
    CacheConfig, CacheStore, DbLocation, canonical_str, db_delete_by_id, db_lookup, file_matches,
};
use crate::error::Result;

/// 参照専用ハンドル。
///
/// `Clone` のコストは `Arc` のカウントアップのみ。スレッド間で自由に共有できる。
#[derive(Clone)]
pub struct CacheReader<E: CacheExtension> {
    pub(crate) store: Arc<CacheStore<E>>,
}

impl<E: CacheExtension> CacheReader<E> {
    pub(crate) fn new(store: Arc<CacheStore<E>>) -> Self {
        Self { store }
    }

    pub fn as_session(config: CacheConfig) -> Result<Self> {
        let db_path = config.db_location.resolve();
        let store = Arc::new(CacheStore::<E>::open(&db_path, config)?);
        Ok(Self { store })
    }

    pub fn onetime(location: DbLocation) -> Result<Self> {
        Self::as_session(CacheConfig {
            db_location: location,
            ..Default::default()
        })
    }

    pub fn open_in_memory() -> Result<Self> {
        let store = Arc::new(CacheStore::<E>::open_in_memory(CacheConfig::default())?);
        Ok(Self { store })
    }

    // -----------------------------------------------------------------------
    // 拡張クレート向け低レベル API
    // -----------------------------------------------------------------------

    pub fn read_conn(&self) -> Result<crate::core::store::ReadConn> {
        self.store.read()
    }

    // -----------------------------------------------------------------------
    // 読み取り API
    // -----------------------------------------------------------------------

    pub fn check(&self, path: &Path) -> Result<bool> {
        let key = match canonical_str(path) {
            Ok(k) => k,
            Err(_) => return Ok(false),
        };

        match db_lookup(&self.store, &key)? {
            None => Ok(false),
            Some((id, stored_hash, stored_mtime)) => {
                if file_matches(&self.store, &stored_hash, stored_mtime, path)? {
                    Ok(true)
                } else {
                    db_delete_by_id(&self.store, id)?;
                    Ok(false)
                }
            }
        }
    }

    /// `check` の一括版。read pool の複数コネクションを使って rayon で並列実行する。
    pub fn check_all(&self, paths: &[&Path]) -> Vec<(PathBuf, Result<bool>)> {
        paths
            .par_iter()
            .map(|p| (p.to_path_buf(), self.check(p)))
            .collect()
    }

    pub fn list_paths(&self) -> Result<Vec<String>> {
        let conn = self.store.read()?;
        let mut stmt = conn.prepare("SELECT path FROM files ORDER BY path")?;
        let paths = stmt
            .query_map([], |r| r.get::<_, String>(0))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(paths)
    }
}

// ---------------------------------------------------------------------------
// CacheRead trait 実装
// ---------------------------------------------------------------------------

impl<E: CacheExtension> CacheRead for CacheReader<E> {
    fn check(&self, path: &Path) -> Result<bool> {
        CacheReader::check(self, path)
    }
    fn check_all(&self, paths: &[&Path]) -> Vec<(PathBuf, Result<bool>)> {
        CacheReader::check_all(self, paths)
    }
    fn list_paths(&self) -> Result<Vec<String>> {
        CacheReader::list_paths(self)
    }
}
