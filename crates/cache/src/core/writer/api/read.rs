use std::path::Path;

use super::super::cache_writer::CacheWriter;
use crate::{ImageCacheEntry, LookupResult, VideoCacheEntry, error::Result};

impl CacheWriter {
    /// 画像ファイルのキャッシュを照会する。[`CacheReader::lookup_image`] と同じ挙動。
    pub fn lookup_image(&self, path: &Path) -> Result<LookupResult<ImageCacheEntry>> {
        self.reader.lookup_image(path)
    }

    /// 動画ファイルのキャッシュを照会する。[`CacheReader::lookup_video`] と同じ挙動。
    pub fn lookup_video(&self, path: &Path) -> Result<LookupResult<VideoCacheEntry>> {
        self.reader.lookup_video(path)
    }

    /// 登録済みファイルパスの一覧を返す。
    pub fn list_paths(&self) -> Result<Vec<String>> {
        self.reader.list_paths()
    }
}
