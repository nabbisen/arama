use std::path::Path;

use arama_repr::codec::{blob_to_vec, vec_to_blob};
use rusqlite::OptionalExtension;

use crate::CacheError;
use crate::error::Result;
use crate::identity::api::{compute, matches_stored};
use crate::store::util::upsert_file_record;
use crate::types::{LookupResult, UpsertVideoRequest, VideoCacheEntry, VideoFeatures};

use super::super::cache_store::CacheStore;

// ---------------------------------------------------------------------------
// 動画ファイル向け API
// ---------------------------------------------------------------------------

impl CacheStore {
    /// 動画ファイルのキャッシュを照会する。挙動は `lookup_image` と同様。
    pub fn lookup_video(&self, file_path: &str) -> Result<LookupResult<VideoCacheEntry>> {
        let path = Path::new(file_path);

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

        let matches = matches_stored(&stored_hash, stored_mtime, path, &self.config.hash_strategy)
            .map_err(|e| CacheError::io(path, e))?;

        if !matches {
            let conn = self.write()?;
            conn.execute("DELETE FROM files WHERE id = ?1", [file_id])?;
            return Ok(LookupResult::Invalidated);
        }

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
                "SELECT clip_vector, wav2vec2_vector FROM video_features WHERE file_id = ?1",
                [file_id],
                |r| Ok((r.get::<_, Vec<u8>>(0)?, r.get::<_, Vec<u8>>(1)?)),
            )
            .optional()?
            .map(|(clip_blob, wav_blob)| -> Result<VideoFeatures> {
                Ok(VideoFeatures {
                    clip_vector: blob_to_vec(&clip_blob)?,
                    wav2vec2_vector: blob_to_vec(&wav_blob)?,
                })
            })
            .transpose()?;

        Ok(LookupResult::Hit(VideoCacheEntry {
            file_path: file_path.to_owned(),
            thumbnail_path,
            features,
        }))
    }

    /// 動画ファイルのキャッシュを登録 / 更新する。
    pub fn upsert_video(&self, req: UpsertVideoRequest) -> Result<()> {
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

        match (&req.clip_vector, &req.wav2vec2_vector) {
            (Some(clip), Some(wav)) => {
                let clip_blob = vec_to_blob(clip.to_owned());
                let wav_blob = vec_to_blob(wav.to_owned());
                conn.execute(
                    "INSERT INTO video_features (file_id, clip_vector, wav2vec2_vector)
                     VALUES (?1, ?2, ?3)
                     ON CONFLICT(file_id) DO UPDATE
                         SET clip_vector     = excluded.clip_vector,
                             wav2vec2_vector = excluded.wav2vec2_vector",
                    rusqlite::params![file_id, clip_blob, wav_blob],
                )?;
            }
            (Some(clip), None) => {
                let clip_blob = vec_to_blob(clip.to_owned());
                conn.execute(
                    "UPDATE video_features SET clip_vector = ?2 WHERE file_id = ?1",
                    rusqlite::params![file_id, clip_blob],
                )?;
            }
            (None, Some(wav)) => {
                let wav_blob = vec_to_blob(wav.to_owned());
                conn.execute(
                    "UPDATE video_features SET wav2vec2_vector = ?2 WHERE file_id = ?1",
                    rusqlite::params![file_id, wav_blob],
                )?;
            }
            (None, None) => {}
        }

        Ok(())
    }
}
