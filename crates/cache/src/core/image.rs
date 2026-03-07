//! `ImageCacheWriter` / `ImageCacheReader` — 画像専用キャッシュハンドル。

use std::path::{Path, PathBuf};
use std::sync::Arc;

use rayon::prelude::*;
use rusqlite::OptionalExtension;

use file_feature_cache::Result;
use file_feature_cache::{CacheConfig, CacheRead, CacheWrite, CacheWriter, DbLocation};

use crate::core::codec::{blob_to_vec, vec_to_blob};
use crate::core::extension::MediaExtension;
use crate::core::thumbnail::{generate_image_thumbnail, thumbnail_dest};
use crate::types::{ImageCacheEntry, ImageFeatures, LookupResult, UpsertImageRequest};

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct ImageCacheConfig {
    pub cache_config: CacheConfig,
}

impl Default for ImageCacheConfig {
    fn default() -> Self {
        Self {
            cache_config: CacheConfig::default(),
        }
    }
}

// ---------------------------------------------------------------------------
// ImageCacheWriter
// ---------------------------------------------------------------------------

/// 画像ファイル専用の更新ハンドル。
///
/// - サムネイルを `image` クレートで自動生成する (224×224 JPEG)
/// - `Clone` のコストは `Arc` のカウントアップのみ
#[derive(Clone)]
pub struct ImageCacheWriter {
    writer: CacheWriter<MediaExtension>,
    config: Arc<ImageCacheConfig>,
}

impl ImageCacheWriter {
    fn build(writer: CacheWriter<MediaExtension>, config: ImageCacheConfig) -> Self {
        Self {
            writer,
            config: Arc::new(config),
        }
    }

    pub fn as_session(config: ImageCacheConfig) -> Result<Self> {
        let writer = CacheWriter::as_session(config.cache_config.clone())?;
        Ok(Self::build(writer, config))
    }

    pub fn onetime(location: DbLocation) -> Result<Self> {
        let writer = CacheWriter::onetime(location)?;
        Ok(Self::build(writer, ImageCacheConfig::default()))
    }

    // -----------------------------------------------------------------------
    // 更新 API
    // -----------------------------------------------------------------------

    pub fn upsert(&self, req: UpsertImageRequest) -> Result<()> {
        let id = self.writer.refresh(&req.path)?;
        let conn = self.writer.write_conn()?;

        if let Some(thumb_dir) = &self.config.cache_config.thumbnail_dir {
            let dest = thumbnail_dest(thumb_dir, id);
            if !dest.exists() {
                generate_image_thumbnail(&req.path, &dest)?;
            }
            conn.execute(
                "INSERT INTO thumbnails (id, thumbnail_path) VALUES (?1, ?2)
                 ON CONFLICT(id) DO UPDATE SET thumbnail_path = excluded.thumbnail_path",
                rusqlite::params![id, dest.to_string_lossy().as_ref()],
            )?;
        }

        if let Some(v) = &req.clip_vector {
            conn.execute(
                "INSERT INTO image_features (id, clip_vector) VALUES (?1, ?2)
                 ON CONFLICT(id) DO UPDATE SET clip_vector = excluded.clip_vector",
                rusqlite::params![id, vec_to_blob(v)],
            )?;
        }

        Ok(())
    }

    /// `upsert` の一括版。各リクエストに対して `(PathBuf, Result<()>)` を返す。
    ///
    /// ## 並列化戦略
    ///
    /// - **fingerprint 計算・サムネイル生成**: rayon で並列実行
    /// - **DB 書き込み**: write pool の制約に従い直列で処理
    ///
    /// 個々のエラーは `Err` として各要素に格納され、他のリクエストの処理は継続する。
    pub fn upsert_all(&self, reqs: Vec<UpsertImageRequest>) -> Vec<(PathBuf, Result<()>)> {
        // ① refresh_all で fingerprint 計算を並列化し、id を取得する
        let paths: Vec<&Path> = reqs.iter().map(|r| r.path.as_path()).collect();
        let id_results = self.writer.refresh_all(&paths);

        // ② id_results と reqs を zip して DB 書き込みを直列処理
        id_results
            .into_iter()
            .zip(reqs)
            .map(|((path, id_result), req)| {
                let result = id_result.and_then(|id| self.write_features(id, &req));
                (path, result)
            })
            .collect()
    }

    pub fn delete(&self, path: &Path) -> Result<bool> {
        self.writer.delete(path)
    }

    pub fn list_paths(&self) -> Result<Vec<String>> {
        self.writer.list_paths()
    }

    pub fn as_reader(&self) -> ImageCacheReader {
        ImageCacheReader {
            reader: self.writer.as_reader(),
            config: self.config.clone(),
        }
    }

    pub fn lookup(&self, path: &Path) -> Result<LookupResult<ImageCacheEntry>> {
        self.as_reader().lookup(path)
    }

    // -----------------------------------------------------------------------
    // 内部
    // -----------------------------------------------------------------------

    /// id が確定した後の拡張テーブル書き込みを担う内部ヘルパー。
    /// upsert と upsert_all の両方から呼ばれる。
    fn write_features(&self, id: i64, req: &UpsertImageRequest) -> Result<()> {
        let conn = self.writer.write_conn()?;

        if let Some(thumb_dir) = &self.config.cache_config.thumbnail_dir {
            let dest = thumbnail_dest(thumb_dir, id);
            if !dest.exists() {
                generate_image_thumbnail(&req.path, &dest)?;
            }
            conn.execute(
                "INSERT INTO thumbnails (id, thumbnail_path) VALUES (?1, ?2)
                 ON CONFLICT(id) DO UPDATE SET thumbnail_path = excluded.thumbnail_path",
                rusqlite::params![id, dest.to_string_lossy().as_ref()],
            )?;
        }

        if let Some(v) = &req.clip_vector {
            conn.execute(
                "INSERT INTO image_features (id, clip_vector) VALUES (?1, ?2)
                 ON CONFLICT(id) DO UPDATE SET clip_vector = excluded.clip_vector",
                rusqlite::params![id, vec_to_blob(v)],
            )?;
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// CacheWrite 実装
// ---------------------------------------------------------------------------

impl CacheWrite for ImageCacheWriter {
    type Reader = ImageCacheReader;

    fn as_session(cache_config: CacheConfig) -> Result<Self> {
        ImageCacheWriter::as_session(ImageCacheConfig { cache_config })
    }
    fn onetime(location: DbLocation) -> Result<Self> {
        ImageCacheWriter::onetime(location)
    }
    fn as_reader(&self) -> ImageCacheReader {
        ImageCacheWriter::as_reader(self)
    }
    fn refresh(&self, path: &Path) -> Result<i64> {
        self.writer.refresh(path)
    }
    fn refresh_all(&self, paths: &[&Path]) -> Vec<(PathBuf, Result<i64>)> {
        self.writer.refresh_all(paths)
    }
    fn delete(&self, path: &Path) -> Result<bool> {
        ImageCacheWriter::delete(self, path)
    }
    fn list_paths(&self) -> Result<Vec<String>> {
        ImageCacheWriter::list_paths(self)
    }
}

// ---------------------------------------------------------------------------
// ImageCacheReader
// ---------------------------------------------------------------------------

/// 画像ファイル専用の参照ハンドル。`Clone` のコストは `Arc` のカウントアップのみ。
#[derive(Clone)]
pub struct ImageCacheReader {
    reader: file_feature_cache::CacheReader<MediaExtension>,
    config: Arc<ImageCacheConfig>,
}

impl ImageCacheReader {
    pub fn as_session(config: ImageCacheConfig) -> Result<Self> {
        let reader = file_feature_cache::CacheReader::as_session(config.cache_config.clone())?;
        Ok(Self {
            reader,
            config: Arc::new(config),
        })
    }

    pub fn onetime(location: DbLocation) -> Result<Self> {
        let reader = file_feature_cache::CacheReader::onetime(location)?;
        Ok(Self {
            reader,
            config: Arc::new(ImageCacheConfig::default()),
        })
    }

    pub fn lookup(&self, path: &Path) -> Result<LookupResult<ImageCacheEntry>> {
        let canonical = match path.canonicalize() {
            Ok(p) => p.to_string_lossy().into_owned(),
            Err(_) => return Ok(LookupResult::Miss),
        };

        let id = {
            let conn = self.reader.read_conn()?;
            conn.query_row(
                "SELECT id FROM files WHERE path = ?1",
                [canonical.as_str()],
                |r| r.get::<_, i64>(0),
            )
            .optional()?
        };
        let id = match id {
            None => return Ok(LookupResult::Miss),
            Some(id) => id,
        };

        if !self.reader.check(path)? {
            return Ok(LookupResult::Invalidated);
        }

        let conn = self.reader.read_conn()?;

        let thumbnail_path = conn
            .query_row(
                "SELECT thumbnail_path FROM thumbnails WHERE id = ?1",
                [id],
                |r| r.get::<_, String>(0),
            )
            .optional()?;

        let features = conn
            .query_row(
                "SELECT clip_vector FROM image_features WHERE id = ?1",
                [id],
                |r| r.get::<_, Vec<u8>>(0),
            )
            .optional()?
            .map(|b| blob_to_vec(&b).map(|v| ImageFeatures { clip_vector: v }))
            .transpose()?;

        Ok(LookupResult::Hit(ImageCacheEntry {
            path: canonical,
            thumbnail_path,
            features,
        }))
    }

    /// `lookup` の一括版。read pool の複数コネクションを使って rayon で並列実行する。
    pub fn lookup_all(
        &self,
        paths: &[&Path],
    ) -> Vec<(PathBuf, Result<LookupResult<ImageCacheEntry>>)> {
        paths
            .par_iter()
            .map(|p| (p.to_path_buf(), self.lookup(p)))
            .collect()
    }

    pub fn check(&self, path: &Path) -> Result<bool> {
        self.reader.check(path)
    }
    pub fn list_paths(&self) -> Result<Vec<String>> {
        self.reader.list_paths()
    }
}

// ---------------------------------------------------------------------------
// CacheRead 実装
// ---------------------------------------------------------------------------

impl CacheRead for ImageCacheReader {
    fn check(&self, path: &Path) -> Result<bool> {
        self.reader.check(path)
    }
    fn check_all(&self, paths: &[&Path]) -> Vec<(PathBuf, Result<bool>)> {
        self.reader.check_all(paths)
    }
    fn list_paths(&self) -> Result<Vec<String>> {
        self.reader.list_paths()
    }
}
