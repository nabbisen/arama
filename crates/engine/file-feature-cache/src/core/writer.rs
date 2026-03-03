//! `CacheWriter<E>` — 参照 + 更新の全権限を持つハンドル。

use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::CacheWrite;
use crate::core::extension::CacheExtension;
use crate::core::reader::CacheReader;
use crate::core::store::{
    CacheConfig, CacheStore, DbLocation, ReadConn, WriteConn, compute_fingerprint, db_delete_by_id,
    db_fetch_file_row, db_upsert_file, file_matches,
};
use crate::error::Result;

/// 参照 + 更新の全権限を持つハンドル。
///
/// - `as_reader()` で参照専用の [`CacheReader<E>`] にダウングレードできる
/// - [`CacheReader<E>`] から `CacheWriter<E>` への昇格はできない
/// - `Clone` は `Arc<CacheStore<E>>` のカウントアップのみで低コスト
#[derive(Clone)]
pub struct CacheWriter<E: CacheExtension> {
    reader: CacheReader<E>,
}

impl<E: CacheExtension> CacheWriter<E> {
    /// 単発・使い捨て用。DB を開いて操作して閉じる想定。
    pub fn oneshot(location: DbLocation, thumbnail_dir: Option<PathBuf>) -> Result<Self> {
        let config = CacheConfig {
            db_location: location,
            thumbnail_dir,
            ..Default::default()
        };
        let path = config.db_location.resolve();
        let inner = Arc::new(CacheStore::<E>::open(&path, config)?);
        Ok(Self {
            reader: CacheReader::new(inner),
        })
    }

    /// 継続使用・rayon 並列処理用セッション。
    pub fn as_session(config: CacheConfig) -> Result<Self> {
        let path = config.db_location.resolve();
        let inner = Arc::new(CacheStore::<E>::open(&path, config)?);
        Ok(Self {
            reader: CacheReader::new(inner),
        })
    }

    /// インメモリ DB を開く (テスト用)。
    pub fn open_in_memory() -> Result<Self> {
        let inner = Arc::new(CacheStore::<E>::open_in_memory(CacheConfig::default())?);
        Ok(Self {
            reader: CacheReader::new(inner),
        })
    }

    /// 参照専用の [`CacheReader<E>`] を返す。
    pub fn as_reader(&self) -> CacheReader<E> {
        self.reader.clone()
    }

    pub(crate) fn store(&self) -> &Arc<CacheStore<E>> {
        &self.reader.store
    }
}

// ---------------------------------------------------------------------------
// 参照 API — CacheReader に委譲
// ---------------------------------------------------------------------------

impl<E: CacheExtension> CacheWriter<E> {
    pub fn list_paths(&self) -> Result<Vec<String>> {
        self.reader.list_paths()
    }

    pub fn check(&self, file_path: &str) -> Result<bool> {
        self.reader.check(file_path)
    }
}

// ---------------------------------------------------------------------------
// 更新 API
// ---------------------------------------------------------------------------

impl<E: CacheExtension> CacheWriter<E> {
    /// ファイルを `files` テーブルに登録し `file_id` を返す。
    ///
    /// 特化クレートはこれを呼んでから拡張テーブルに書き込む。
    pub fn upsert_file(&self, file_path: &str) -> Result<i64> {
        let path = Path::new(file_path);
        let fp = compute_fingerprint(self.store(), path)?;
        let conn = self.store().write()?;
        db_upsert_file(&conn, file_path, &fp.hash, fp.mtime_ns).map_err(Into::into)
    }

    /// ファイルパスに紐付くキャッシュを全て削除する (CASCADE で拡張テーブルも削除)。
    /// 戻り値: 対象レコードが存在した場合 `true`
    pub fn delete(&self, file_path: &str) -> Result<bool> {
        let conn = self.store().write()?;
        let n = conn.execute("DELETE FROM files WHERE file_path = ?1", [file_path])?;
        Ok(n > 0)
    }

    /// ファイルの現在の状態を確認し、変更されていれば DB から削除して `false` を返す。
    pub fn verify_or_invalidate(&self, file_path: &str) -> Result<bool> {
        let path = Path::new(file_path);
        match db_fetch_file_row(self.store(), file_path)? {
            None => Ok(true),
            Some((file_id, stored_hash, stored_mtime)) => {
                if file_matches(self.store(), &stored_hash, stored_mtime, path)? {
                    Ok(true)
                } else {
                    db_delete_by_id(self.store(), file_id)?;
                    Ok(false)
                }
            }
        }
    }

    /// 拡張テーブルへの書き込みに使う write 接続を取得する。
    pub fn write_conn(&self) -> Result<WriteConn> {
        self.store().write()
    }

    /// 拡張テーブルからの読み取りに使う read 接続を取得する。
    pub fn read_conn(&self) -> Result<ReadConn> {
        self.store().read()
    }

    /// `thumbnail_dir` が設定されている場合、サムネイルの保存パスを返す。
    ///
    /// `<thumbnail_dir>/<file_id>.<ext>` の形式。
    /// `thumbnail_dir` が未設定の場合は `None`。
    pub fn thumbnail_path(&self, file_id: i64, ext: &str) -> Option<PathBuf> {
        self.store()
            .config
            .thumbnail_dir
            .as_ref()
            .map(|dir| dir.join(format!("{file_id}.{ext}")))
    }
}

// ---------------------------------------------------------------------------
// CacheWrite trait 実装
// ---------------------------------------------------------------------------

impl<E: CacheExtension> CacheWrite for CacheWriter<E> {
    type Reader = CacheReader<E>;
    type Config = CacheConfig;

    fn as_session(config: CacheConfig) -> Result<Self> {
        CacheWriter::as_session(config)
    }

    fn oneshot(location: DbLocation) -> Result<Self> {
        CacheWriter::oneshot(location, None)
    }

    fn as_reader(&self) -> CacheReader<E> {
        CacheWriter::as_reader(self)
    }

    fn delete(&self, file_path: &str) -> Result<bool> {
        CacheWriter::delete(self, file_path)
    }

    fn verify_or_invalidate(&self, file_path: &str) -> Result<bool> {
        CacheWriter::verify_or_invalidate(self, file_path)
    }

    fn list_paths(&self) -> Result<Vec<String>> {
        CacheWriter::list_paths(self)
    }
}
