//! `CacheWriter` — 参照 + 更新の全権限を持つハンドル。

use std::path::Path;
use std::sync::Arc;

pub mod api;
pub mod cache_writer;

use crate::CacheReader;
use crate::error::{Result, cache_error::CacheError};
use crate::identity::api::compute;
use crate::reader::util::file_matches;
use crate::store::cache_store::CacheStore;
use crate::store::helper::reader::db_fetch_file_row;
use crate::store::helper::writer::{
    db_delete_by_id, db_upsert_file, db_upsert_image_features, db_upsert_thumbnail,
    db_upsert_video_features,
};
use crate::store::path::resolve_db_path;
use crate::types::{
    ImageCacheEntry, LookupResult, UpsertImageRequest, UpsertVideoRequest, VideoCacheEntry,
};

// CacheConfig は inner で定義し、ここから re-export する
pub use crate::config::cache_config::CacheConfig;

// ---------------------------------------------------------------------------
// CacheWriter
// ---------------------------------------------------------------------------

/// 参照 + 更新の全権限を持つハンドル。
///
/// - lookup / list は [`CacheReader`] に委譲する (コードの重複なし)
/// - [`as_reader`] で参照専用の [`CacheReader`] にダウングレードできる
/// - [`CacheReader`] から `CacheWriter` への昇格はできない
///
/// `Clone` は `Arc<CacheStore>` のカウントアップのみで低コスト。
///
/// [`as_reader`]: CacheWriter::as_reader
#[derive(Clone)]
pub struct CacheWriter {
    reader: CacheReader,
}

impl CacheWriter {
    /// デフォルト設定で DB を開く。
    ///
    /// DB パスは自動解決される (詳細は [`CacheConfig`] を参照)。
    /// 設定を変えたい場合は [`open_with_config`] を使う。
    ///
    /// [`open_with_config`]: CacheWriter::open_with_config
    pub fn open() -> Result<Self> {
        Self::open_with_config(CacheConfig::default())
    }

    /// カスタム設定で DB を開く。
    ///
    /// DB パスは自動解決される (詳細は [`CacheConfig`] を参照)。
    pub fn open_with_config(config: CacheConfig) -> Result<Self> {
        let path = resolve_db_path();
        let inner = Arc::new(CacheStore::open(&path, config)?);
        Ok(Self {
            reader: CacheReader::new(inner),
        })
    }

    /// インメモリ DB を開く (テスト用)。
    pub fn open_in_memory() -> Result<Self> {
        Self::open_in_memory_with_config(CacheConfig::default())
    }

    /// 設定付きインメモリ DB を開く (テスト用)。
    pub fn open_in_memory_with_config(config: CacheConfig) -> Result<Self> {
        let inner = Arc::new(CacheStore::open_in_memory(config)?);
        Ok(Self {
            reader: CacheReader::new(inner),
        })
    }

    /// 参照専用の [`CacheReader`] を返す。
    ///
    /// 内部の `Arc<CacheStore>` を共有するため、コスト・DB 接続ともに追加消費なし。
    /// lookup のみ必要な箇所に渡すことで権限を制限できる。
    pub fn as_reader(&self) -> CacheReader {
        self.reader.clone()
    }

    fn inner(&self) -> &Arc<CacheStore> {
        &self.reader.inner
    }
}

// ---------------------------------------------------------------------------
// 参照 API — CacheReader に委譲
// ---------------------------------------------------------------------------

impl CacheWriter {
    /// 画像ファイルのキャッシュを照会する。[`CacheReader::lookup_image`] と同じ挙動。
    pub fn lookup_image(&self, file_path: &str) -> Result<LookupResult<ImageCacheEntry>> {
        self.reader.lookup_image(file_path)
    }

    /// 動画ファイルのキャッシュを照会する。[`CacheReader::lookup_video`] と同じ挙動。
    pub fn lookup_video(&self, file_path: &str) -> Result<LookupResult<VideoCacheEntry>> {
        self.reader.lookup_video(file_path)
    }

    /// 登録済みファイルパスの一覧を返す。
    pub fn list_paths(&self) -> Result<Vec<String>> {
        self.reader.list_paths()
    }
}

// ---------------------------------------------------------------------------
// 更新 API
// ---------------------------------------------------------------------------

impl CacheWriter {
    /// 画像ファイルのキャッシュを登録 / 更新する。
    ///
    /// `file_path` のファイルから識別情報を自動計算して DB に保存する。
    /// `thumbnail_path` / `clip_vector` が `None` の場合、既存の値を上書きしない (部分更新)。
    pub fn upsert_image(&self, req: UpsertImageRequest) -> Result<()> {
        let path = Path::new(&req.file_path);
        let fp = compute(path, &self.inner().config.hash_strategy)
            .map_err(|e| CacheError::io(path, e))?;

        let conn = self.inner().write()?;
        let file_id = db_upsert_file(&conn, &req.file_path, &fp.hash, fp.mtime_ns)?;

        if let Some(ref p) = req.thumbnail_path {
            db_upsert_thumbnail(&conn, file_id, p)?;
        }
        if let Some(ref v) = req.clip_vector {
            db_upsert_image_features(&conn, file_id, v)?;
        }
        Ok(())
    }

    /// 動画ファイルのキャッシュを登録 / 更新する。
    pub fn upsert_video(&self, req: UpsertVideoRequest) -> Result<()> {
        let path = Path::new(&req.file_path);
        let fp = compute(path, &self.inner().config.hash_strategy)
            .map_err(|e| CacheError::io(path, e))?;

        let conn = self.inner().write()?;
        let file_id = db_upsert_file(&conn, &req.file_path, &fp.hash, fp.mtime_ns)?;

        if let Some(ref p) = req.thumbnail_path {
            db_upsert_thumbnail(&conn, file_id, p)?;
        }
        db_upsert_video_features(
            &conn,
            file_id,
            req.clip_vector.as_ref(),
            req.wav2vec2_vector.as_ref(),
        )?;
        Ok(())
    }

    /// ファイルパスに紐付くキャッシュを全て削除する。
    /// 戻り値: 対象レコードが存在した場合 `true`
    pub fn delete(&self, file_path: &str) -> Result<bool> {
        let conn = self.inner().write()?;
        let n = conn.execute("DELETE FROM files WHERE file_path = ?1", [file_path])?;
        Ok(n > 0)
    }

    /// ファイルの現在の状態を確認し、変更されていれば DB から削除して `false` を返す。
    /// 変更なし (またはレコード未存在) の場合は `true` を返す。
    ///
    /// 大量ファイルの一括検証など、キャッシュ保守スキャンに使う。
    pub fn verify_or_invalidate(&self, file_path: &str) -> Result<bool> {
        let path = Path::new(file_path);
        match db_fetch_file_row(self.inner(), file_path)? {
            None => Ok(true),
            Some((file_id, stored_hash, stored_mtime)) => {
                if file_matches(self.inner(), &stored_hash, stored_mtime, path)? {
                    Ok(true)
                } else {
                    db_delete_by_id(self.inner(), file_id)?;
                    Ok(false)
                }
            }
        }
    }
}
