use std::path::Path;

use super::super::super::{cache_reader::CacheReader, util::file_matches};
use crate::core::store::helper::reader::{
    db_fetch_file_row, db_fetch_image_features, db_fetch_thumbnail, db_fetch_video_features,
};
use crate::core::store::helper::writer::db_delete_by_id;
use crate::error::Result;
use crate::types::{ImageCacheEntry, LookupResult, VideoCacheEntry};

impl CacheReader {
    /// 画像ファイルのキャッシュを照会する。
    ///
    /// ファイルを読み取り DB の識別情報と比較する。
    /// 変更が検出された場合は古いレコードを内部で削除し `Invalidated` を返す。
    pub fn lookup_image(&self, path: &Path) -> Result<LookupResult<ImageCacheEntry>> {
        let path = path.canonicalize().expect("failed to canonicalize path");
        let path_str = path.to_string_lossy().to_string();

        let (file_id, stored_hash, stored_mtime) = match db_fetch_file_row(&self.store, &path_str)?
        {
            None => return Ok(LookupResult::Miss),
            Some(r) => r,
        };

        if !file_matches(&self.store, &stored_hash, stored_mtime, &path)? {
            db_delete_by_id(&self.store, file_id)?;
            return Ok(LookupResult::Invalidated);
        }

        let conn = self.store.read()?;
        let thumbnail_path = db_fetch_thumbnail(&conn, file_id)?;
        let features = db_fetch_image_features(&conn, file_id)?;

        Ok(LookupResult::Hit(ImageCacheEntry {
            file_path: path_str,
            thumbnail_path,
            features,
        }))
    }

    /// 動画ファイルのキャッシュを照会する。挙動は `lookup_image` と同様。
    pub fn lookup_video(&self, path: &Path) -> Result<LookupResult<VideoCacheEntry>> {
        let path = path.canonicalize().expect("failed to canonicalize path");
        let path_str = path.to_string_lossy().to_string();

        let (file_id, stored_hash, stored_mtime) = match db_fetch_file_row(&self.store, &path_str)?
        {
            None => return Ok(LookupResult::Miss),
            Some(r) => r,
        };

        if !file_matches(&self.store, &stored_hash, stored_mtime, &path)? {
            db_delete_by_id(&self.store, file_id)?;
            return Ok(LookupResult::Invalidated);
        }

        let conn = self.store.read()?;
        let thumbnail_path = db_fetch_thumbnail(&conn, file_id)?;
        let features = db_fetch_video_features(&conn, file_id)?;

        Ok(LookupResult::Hit(VideoCacheEntry {
            file_path: path_str,
            thumbnail_path,
            features,
        }))
    }

    /// 登録済みファイルパスの一覧を返す。
    pub fn list_paths(&self) -> Result<Vec<String>> {
        let conn = self.store.read()?;
        let mut stmt = conn.prepare("SELECT file_path FROM files ORDER BY file_path")?;
        let paths = stmt
            .query_map([], |r| r.get::<_, String>(0))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(paths)
    }
}
