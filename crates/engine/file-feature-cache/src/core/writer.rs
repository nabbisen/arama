//! `CacheWriter<E>` — 参照 + 更新の全権限を持つハンドル。

use std::path::{Path, PathBuf};
use std::sync::Arc;

use rayon::prelude::*;

use crate::core::extension::CacheExtension;
use crate::core::identity::FileFingerprint;
use crate::core::reader::CacheReader;
use crate::core::store::{
    CacheConfig, CacheStore, DbLocation, ReadConn, WriteConn, canonical_str, compute_fingerprint,
    db_delete_by_id, db_delete_by_key, db_insert, db_lookup, file_matches,
};
use crate::error::Result;
use crate::{CacheRead, CacheWrite};

/// 参照 + 更新の全権限を持つハンドル。
///
/// - `as_reader()` で参照専用の [`CacheReader<E>`] にダウングレードできる
/// - `Clone` のコストは `Arc` のカウントアップのみ
#[derive(Clone)]
pub struct CacheWriter<E: CacheExtension> {
    reader: CacheReader<E>,
}

impl<E: CacheExtension> CacheWriter<E> {
    // -----------------------------------------------------------------------
    // ファクトリ
    // -----------------------------------------------------------------------

    pub fn as_session(config: CacheConfig) -> Result<Self> {
        let db_path = config.db_location.resolve();
        let store = Arc::new(CacheStore::<E>::open(&db_path, config)?);
        Ok(Self {
            reader: CacheReader::new(store),
        })
    }

    pub fn onetime(location: DbLocation) -> Result<Self> {
        Self::as_session(CacheConfig {
            db_location: location,
            ..Default::default()
        })
    }

    pub fn open_in_memory() -> Result<Self> {
        let store = Arc::new(CacheStore::<E>::open_in_memory(CacheConfig::default())?);
        Ok(Self {
            reader: CacheReader::new(store),
        })
    }

    // -----------------------------------------------------------------------
    // 参照 API — CacheReader に委譲
    // -----------------------------------------------------------------------

    pub fn as_reader(&self) -> CacheReader<E> {
        self.reader.clone()
    }

    pub fn check(&self, path: &Path) -> Result<bool> {
        self.reader.check(path)
    }

    pub fn check_all(&self, paths: &[&Path]) -> Vec<(PathBuf, Result<bool>)> {
        self.reader.check_all(paths)
    }

    pub fn list_paths(&self) -> Result<Vec<String>> {
        self.reader.list_paths()
    }

    // -----------------------------------------------------------------------
    // 更新 API
    // -----------------------------------------------------------------------

    pub fn refresh(&self, path: &Path) -> Result<i64> {
        let key = canonical_str(path)?;

        match db_lookup(self.store(), &key)? {
            None => {
                let fp = compute_fingerprint(self.store(), path)?;
                db_insert(self.store(), &key, &fp.hash, fp.mtime_ns)
            }
            Some((id, stored_hash, stored_mtime)) => {
                if file_matches(self.store(), &stored_hash, stored_mtime, path)? {
                    Ok(id)
                } else {
                    db_delete_by_id(self.store(), id)?;
                    let fp = compute_fingerprint(self.store(), path)?;
                    db_insert(self.store(), &key, &fp.hash, fp.mtime_ns)
                }
            }
        }
    }

    /// `refresh` の一括版。
    ///
    /// ## 並列化戦略
    ///
    /// - **fingerprint 計算** (ファイル読み取り + SHA-256): rayon で並列実行
    /// - **DB 書き込み**: write pool の制約 (max_size = 1) に従い直列で処理
    ///
    /// 個々のエラーは `Err` として各要素に格納され、他のパスの処理は継続する。
    pub fn refresh_all(&self, paths: &[&Path]) -> Vec<(PathBuf, Result<i64>)> {
        // ① canonicalize + fingerprint 計算を並列実行
        //    失敗した要素は Err のまま次フェーズに渡す
        let fingerprints: Vec<(PathBuf, Result<(String, FileFingerprint)>)> = paths
            .par_iter()
            .map(|p| {
                let result = canonical_str(p)
                    .and_then(|key| compute_fingerprint(self.store(), p).map(|fp| (key, fp)));
                (p.to_path_buf(), result)
            })
            .collect();

        // ② DB 書き込みは直列
        fingerprints
            .into_iter()
            .map(|(path, fp_result)| {
                let id_result = fp_result
                    .and_then(|(key, fp)| self.refresh_with_key_and_fingerprint(&path, &key, fp));
                (path, id_result)
            })
            .collect()
    }

    pub fn delete(&self, path: &Path) -> Result<bool> {
        let key = canonical_str(path)?;
        db_delete_by_key(self.store(), &key)
    }

    // -----------------------------------------------------------------------
    // 拡張クレート向け低レベル API
    // -----------------------------------------------------------------------

    pub fn write_conn(&self) -> Result<WriteConn> {
        self.store().write()
    }

    pub fn read_conn(&self) -> Result<ReadConn> {
        self.store().read()
    }

    pub fn thumbnail_path(&self, id: i64, ext: &str) -> Option<PathBuf> {
        self.store()
            .config
            .thumbnail_dir
            .as_ref()
            .map(|dir| dir.join(format!("{id}.{ext}")))
    }

    // -----------------------------------------------------------------------
    // 内部
    // -----------------------------------------------------------------------

    pub(crate) fn store(&self) -> &Arc<CacheStore<E>> {
        &self.reader.store
    }

    /// fingerprint が計算済みの状態で DB 操作だけを行う内部ヘルパー。
    /// refresh_all の直列フェーズから呼ばれる。
    fn refresh_with_key_and_fingerprint(
        &self,
        path: &Path,
        key: &str,
        fp: FileFingerprint,
    ) -> Result<i64> {
        match db_lookup(self.store(), key)? {
            None => db_insert(self.store(), key, &fp.hash, fp.mtime_ns),
            Some((id, stored_hash, stored_mtime)) => {
                if file_matches(self.store(), &stored_hash, stored_mtime, path)? {
                    Ok(id)
                } else {
                    db_delete_by_id(self.store(), id)?;
                    db_insert(self.store(), key, &fp.hash, fp.mtime_ns)
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// CacheWrite trait 実装
// ---------------------------------------------------------------------------

impl<E: CacheExtension> CacheWrite for CacheWriter<E> {
    type Reader = CacheReader<E>;

    fn as_session(config: CacheConfig) -> Result<Self> {
        CacheWriter::as_session(config)
    }
    fn onetime(location: DbLocation) -> Result<Self> {
        CacheWriter::onetime(location)
    }
    fn as_reader(&self) -> CacheReader<E> {
        CacheWriter::as_reader(self)
    }
    fn refresh(&self, path: &Path) -> Result<i64> {
        CacheWriter::refresh(self, path)
    }
    fn refresh_all(&self, paths: &[&Path]) -> Vec<(PathBuf, Result<i64>)> {
        CacheWriter::refresh_all(self, paths)
    }
    fn delete(&self, path: &Path) -> Result<bool> {
        CacheWriter::delete(self, path)
    }
    fn list_paths(&self) -> Result<Vec<String>> {
        CacheWriter::list_paths(self)
    }
}

// ---------------------------------------------------------------------------
// CacheRead 委譲実装
// ---------------------------------------------------------------------------

impl<E: CacheExtension> CacheRead for CacheWriter<E> {
    fn check(&self, path: &Path) -> Result<bool> {
        self.reader.check(path)
    }
    fn check_all(&self, paths: &[&Path]) -> Vec<(PathBuf, Result<bool>)> {
        self.reader.check_all(paths)
    }
    fn list_paths(&self) -> Result<Vec<String>> {
        self.reader.list_paths()
    }
}
