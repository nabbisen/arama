use arama_repr::codec::vec_to_blob;

use crate::{core::store::cache_store::CacheStore, error::Result, types::WriteConn};

/// `files` テーブルを UPSERT し、挿入 / 更新後の `id` を返す。
pub fn db_upsert_file(
    conn: &WriteConn,
    file_path: &str,
    hash: &str,
    mtime_ns: Option<i64>,
) -> rusqlite::Result<i64> {
    conn.execute(
        "INSERT INTO files (file_path, file_hash, mtime_ns, updated_at)
         VALUES (?1, ?2, ?3, strftime('%s','now'))
         ON CONFLICT(file_path) DO UPDATE
             SET file_hash  = excluded.file_hash,
                 mtime_ns   = excluded.mtime_ns,
                 updated_at = strftime('%s','now')",
        rusqlite::params![file_path, hash, mtime_ns],
    )?;
    conn.query_row(
        "SELECT id FROM files WHERE file_path = ?1",
        [file_path],
        |r| r.get::<_, i64>(0),
    )
}

/// `thumbnails` テーブルを UPSERT する。
pub fn db_upsert_thumbnail(conn: &WriteConn, file_id: i64, path: &str) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO thumbnails (file_id, thumbnail_path) VALUES (?1, ?2)
         ON CONFLICT(file_id) DO UPDATE SET thumbnail_path = excluded.thumbnail_path",
        rusqlite::params![file_id, path],
    )?;
    Ok(())
}

/// `image_features` テーブルを UPSERT する。
pub fn db_upsert_image_features(
    conn: &WriteConn,
    file_id: i64,
    clip: &[f32],
) -> rusqlite::Result<()> {
    let blob = vec_to_blob(clip.to_vec());
    conn.execute(
        "INSERT INTO image_features (file_id, clip_vector) VALUES (?1, ?2)
         ON CONFLICT(file_id) DO UPDATE SET clip_vector = excluded.clip_vector",
        rusqlite::params![file_id, blob],
    )?;
    Ok(())
}

/// `video_features` テーブルを UPSERT する。`None` の場合は既存値を保持する。
pub fn db_upsert_video_features(
    conn: &WriteConn,
    file_id: i64,
    clip: Option<&Vec<f32>>,
    wav: Option<&Vec<f32>>,
) -> rusqlite::Result<()> {
    match (clip, wav) {
        (Some(c), Some(w)) => {
            conn.execute(
                "INSERT INTO video_features (file_id, clip_vector, wav2vec2_vector)
                 VALUES (?1, ?2, ?3)
                 ON CONFLICT(file_id) DO UPDATE
                     SET clip_vector     = excluded.clip_vector,
                         wav2vec2_vector = excluded.wav2vec2_vector",
                rusqlite::params![
                    file_id,
                    vec_to_blob(c.to_owned()),
                    vec_to_blob(w.to_owned())
                ],
            )?;
        }
        (Some(c), None) => {
            conn.execute(
                "UPDATE video_features SET clip_vector = ?2 WHERE file_id = ?1",
                rusqlite::params![file_id, vec_to_blob(c.to_owned())],
            )?;
        }
        (None, Some(w)) => {
            conn.execute(
                "UPDATE video_features SET wav2vec2_vector = ?2 WHERE file_id = ?1",
                rusqlite::params![file_id, vec_to_blob(w.to_owned())],
            )?;
        }
        (None, None) => {}
    }
    Ok(())
}

/// `files` テーブルから id 指定で削除する (CASCADE で子テーブルも削除)。
pub fn db_delete_by_id(inner: &CacheStore, file_id: i64) -> Result<()> {
    let conn = inner.write()?;
    conn.execute("DELETE FROM files WHERE id = ?1", [file_id])?;
    Ok(())
}
