use std::path::Path;

use super::super::cache_writer::CacheWriter;
use crate::error::{Result, cache_error::CacheError};
use crate::identity::api::compute;
use crate::reader::util::file_matches;
use crate::store::helper::reader::db_fetch_file_row;
use crate::store::helper::writer::{
    db_delete_by_id, db_upsert_file, db_upsert_image_features, db_upsert_thumbnail,
    db_upsert_video_features,
};
use crate::types::{UpsertImageRequest, UpsertVideoRequest};

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
