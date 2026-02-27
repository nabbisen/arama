use super::super::super::cache_writer::CacheWriter;
use crate::{ImageCacheEntry, LookupResult, VideoCacheEntry, error::Result};

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
