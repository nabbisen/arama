use std::path::Path;

use arama_repr::codec::{blob_to_vec, vec_to_blob};
use rusqlite::OptionalExtension;

use crate::CacheError;
use crate::error::Result;
use crate::identity::api::{compute, matches_stored};
use crate::store::util::upsert_file_record;
use crate::types::{ImageCacheEntry, ImageFeatures, LookupResult, UpsertImageRequest};

use super::super::cache_store::CacheStore;

// ---------------------------------------------------------------------------
// 画像ファイル向け API
// ---------------------------------------------------------------------------

impl CacheStore {
    /// 画像ファイルのキャッシュを照会する。
    ///
    /// `file_path` のファイルを実際に読み取り、DB に保存された識別情報と比較する。
    ///
    /// - `Miss`        : DB にレコードなし
    /// - `Invalidated` : ファイルが変更されていた → 古いレコードを削除して返す
    /// - `Hit`         : ファイルが同一 → エントリを返す
    ///
    /// ## mtime fast path (大ファイル + `SizeAdaptive` 時)
    ///
    /// mtime が一致すればハッシュ計算をスキップするため、
    /// 変更のないファイルへの繰り返し lookup は I/O をほぼ発生させない。
    pub fn lookup_image(&self, file_path: &str) -> Result<LookupResult<ImageCacheEntry>> {
        let path = Path::new(file_path);

        // --- フェーズ 1: DB から保存済み識別情報を取得 (read) ---
        let row = {
            let conn = self.read()?;
            conn.query_row(
                "SELECT id, file_hash, mtime_ns FROM files WHERE file_path = ?1",
                [file_path],
                |r| {
                    Ok((
                        r.get::<_, i64>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, Option<i64>>(2)?,
                    ))
                },
            )
            .optional()?
        };

        let (file_id, stored_hash, stored_mtime) = match row {
            None => return Ok(LookupResult::Miss),
            Some(r) => r,
        };

        // --- フェーズ 2: 現在のファイルと比較 ---
        let matches = matches_stored(&stored_hash, stored_mtime, path, &self.config.hash_strategy)
            .map_err(|e| CacheError::io(path, e))?;

        if !matches {
            let conn = self.write()?;
            conn.execute("DELETE FROM files WHERE id = ?1", [file_id])?;
            return Ok(LookupResult::Invalidated);
        }

        // --- フェーズ 3: キャッシュデータを取得 (read) ---
        let conn = self.read()?;

        let thumbnail_path = conn
            .query_row(
                "SELECT thumbnail_path FROM thumbnails WHERE file_id = ?1",
                [file_id],
                |r| r.get::<_, String>(0),
            )
            .optional()?;

        let features = conn
            .query_row(
                "SELECT clip_vector FROM image_features WHERE file_id = ?1",
                [file_id],
                |r| r.get::<_, Vec<u8>>(0),
            )
            .optional()?
            .map(|blob| -> Result<ImageFeatures> {
                Ok(ImageFeatures {
                    clip_vector: blob_to_vec(&blob)?,
                })
            })
            .transpose()?;

        Ok(LookupResult::Hit(ImageCacheEntry {
            file_path: file_path.to_owned(),
            thumbnail_path,
            features,
        }))
    }

    /// 画像ファイルのキャッシュを登録 / 更新する。
    ///
    /// `file_path` のファイルから識別情報 (hash / mtime) を自動計算して DB に保存する。
    /// `thumbnail_path` / `clip_vector` が `None` の場合、既存の値を上書きしない (部分更新)。
    pub fn upsert_image(&self, req: UpsertImageRequest) -> Result<()> {
        let path = Path::new(&req.file_path);
        let fp = compute(path, &self.config.hash_strategy).map_err(|e| CacheError::io(path, e))?;

        let conn = self.write()?;
        let file_id = upsert_file_record(&conn, &req.file_path, &fp.hash, fp.mtime_ns)?;

        if let Some(ref p) = req.thumbnail_path {
            conn.execute(
                "INSERT INTO thumbnails (file_id, thumbnail_path) VALUES (?1, ?2)
                 ON CONFLICT(file_id) DO UPDATE SET thumbnail_path = excluded.thumbnail_path",
                rusqlite::params![file_id, p],
            )?;
        }

        if let Some(ref vec) = req.clip_vector {
            let blob = vec_to_blob(vec.to_owned());
            conn.execute(
                "INSERT INTO image_features (file_id, clip_vector) VALUES (?1, ?2)
                 ON CONFLICT(file_id) DO UPDATE SET clip_vector = excluded.clip_vector",
                rusqlite::params![file_id, blob],
            )?;
        }

        Ok(())
    }
}
