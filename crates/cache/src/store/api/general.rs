use std::path::Path;

use rusqlite::OptionalExtension;

use super::super::cache_store::CacheStore;
use crate::{CacheError, error::Result, identity::api::matches_stored};

// ---------------------------------------------------------------------------
// 汎用 API
// ---------------------------------------------------------------------------

impl CacheStore {
    /// ファイルパスに紐付くキャッシュを全て削除する。
    /// 戻り値: 対象レコードが存在した場合 `true`
    pub fn delete(&self, file_path: &str) -> Result<bool> {
        let conn = self.write()?;
        let affected = conn.execute("DELETE FROM files WHERE file_path = ?1", [file_path])?;
        Ok(affected > 0)
    }

    /// ファイルの現在の状態を確認し、変更されていれば DB から削除して `false` を返す。
    /// 変更なし (またはレコード未存在) の場合は `true` を返す。
    pub fn verify_or_invalidate(&self, file_path: &str) -> Result<bool> {
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

        match row {
            None => Ok(true),
            Some((file_id, stored_hash, stored_mtime)) => {
                let matches =
                    matches_stored(&stored_hash, stored_mtime, path, &self.config.hash_strategy)
                        .map_err(|e| CacheError::io(path, e))?;
                if matches {
                    Ok(true)
                } else {
                    let conn = self.write()?;
                    conn.execute("DELETE FROM files WHERE id = ?1", [file_id])?;
                    Ok(false)
                }
            }
        }
    }

    /// 登録済みファイルパスの一覧を返す
    pub fn list_paths(&self) -> Result<Vec<String>> {
        let conn = self.read()?;
        let mut stmt = conn.prepare("SELECT file_path FROM files ORDER BY file_path")?;
        let paths = stmt
            .query_map([], |r| r.get::<_, String>(0))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(paths)
    }
}
