use arama_repr::codec::blob_to_vec;
use rusqlite::OptionalExtension;

use crate::core::store::cache_store::CacheStore;
use crate::error::Result;
use crate::types::{ImageFeatures, ReadConn, VideoFeatures};

/// `files` テーブルから `(id, file_hash, mtime_ns)` を取得する。
pub fn db_fetch_file_row(
    inner: &CacheStore,
    file_path: &str,
) -> Result<Option<(i64, String, Option<i64>)>> {
    let conn = inner.read()?;
    let row = conn
        .query_row(
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
        .optional()?;
    Ok(row)
}

/// `thumbnails` テーブルからサムネイルパスを取得する。
pub fn db_fetch_thumbnail(conn: &ReadConn, file_id: i64) -> rusqlite::Result<Option<String>> {
    conn.query_row(
        "SELECT thumbnail_path FROM thumbnails WHERE file_id = ?1",
        [file_id],
        |r| r.get::<_, String>(0),
    )
    .optional()
}

/// `image_features` テーブルから画像特徴量を取得する。
pub fn db_fetch_image_features(conn: &ReadConn, file_id: i64) -> Result<Option<ImageFeatures>> {
    let blob = conn
        .query_row(
            "SELECT clip_vector FROM image_features WHERE file_id = ?1",
            [file_id],
            |r| r.get::<_, Vec<u8>>(0),
        )
        .optional()?;

    blob.map(|b| {
        Ok(ImageFeatures {
            clip_vector: blob_to_vec(&b)?,
        })
    })
    .transpose()
}

/// `video_features` テーブルから動画特徴量を取得する。
pub fn db_fetch_video_features(conn: &ReadConn, file_id: i64) -> Result<Option<VideoFeatures>> {
    let row = conn
        .query_row(
            "SELECT clip_vector, wav2vec2_vector FROM video_features WHERE file_id = ?1",
            [file_id],
            |r| Ok((r.get::<_, Vec<u8>>(0)?, r.get::<_, Vec<u8>>(1)?)),
        )
        .optional()?;

    row.map(|(clip, wav)| {
        Ok(VideoFeatures {
            clip_vector: blob_to_vec(&clip)?,
            wav2vec2_vector: blob_to_vec(&wav)?,
        })
    })
    .transpose()
}
