use std::path::Path;

use super::super::cache_writer::CacheWriter;
use crate::core::identity::api::compute;
use crate::core::reader::util::file_matches;
use crate::core::store::helper::reader::db_fetch_file_row;
use crate::core::store::helper::writer::{
    db_delete_by_id, db_upsert_file, db_upsert_image_features, db_upsert_thumbnail,
    db_upsert_video_features,
};
use crate::error::{CacheError, Result};
use crate::types::{UpsertImageRequest, UpsertVideoRequest};

impl CacheWriter {
    /// 画像ファイルのキャッシュを登録 / 更新する。
    ///
    /// `file_path` のファイルから識別情報を自動計算して DB に保存する。
    /// `thumbnail_path` / `clip_vector` が `None` の場合、既存の値を上書きしない (部分更新)。
    pub fn upsert_image(&self, req: UpsertImageRequest) -> Result<()> {
        let path = Path::new(&req.file_path);
        let fp = compute(path, &self.store().config.hash_strategy)
            .map_err(|e| CacheError::io(path, e))?;

        let conn = self.store().write()?;
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
        let fp = compute(path, &self.store().config.hash_strategy)
            .map_err(|e| CacheError::io(path, e))?;

        let conn = self.store().write()?;
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
    pub fn delete(&self, path: &Path) -> Result<bool> {
        let path_str = path
            .canonicalize()
            .expect("failed to canonicalize path")
            .to_string_lossy()
            .to_string();
        let conn = self.store().write()?;
        let n = conn.execute("DELETE FROM files WHERE file_path = ?1", [path_str])?;
        Ok(n > 0)
    }

    /// ファイルの現在の状態を確認し、変更されていれば DB から削除して `false` を返す。
    /// 変更なし (またはレコード未存在) の場合は `true` を返す。
    ///
    /// 大量ファイルの一括検証など、キャッシュ保守スキャンに使う。
    pub fn verify_or_invalidate(&self, path: &Path) -> Result<bool> {
        let path_str = path
            .canonicalize()
            .expect("failed to canonicalize path")
            .to_string_lossy()
            .to_string();

        match db_fetch_file_row(self.store(), &path_str)? {
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
}
