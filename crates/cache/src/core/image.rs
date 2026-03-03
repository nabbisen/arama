//! `ImageCacheWriter` / `ImageCacheReader` — 画像専用キャッシュハンドル。

use std::sync::Arc;

use arama_repr::codec::{blob_to_vec, vec_to_blob};
use file_feature_cache::error::CacheError;
use file_feature_cache::{CacheConfig, CacheRead, CacheWrite, CacheWriter, DbLocation, Result};
use rusqlite::{OptionalExtension, params};

use crate::core::extension::MediaExtension;
use crate::core::thumbnail::{generate_image_thumbnail, thumbnail_dest};
use crate::types::{ImageCacheEntry, ImageFeatures, LookupResult, UpsertImageRequest};

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

/// [`ImageCacheWriter::as_session`] に渡す設定。
#[derive(Debug, Clone)]
pub struct ImageCacheConfig {
    /// エンジン設定 (DB パス・コネクション数・サムネイルディレクトリ)。
    pub cache: CacheConfig,
    /// サムネイルを自動生成するか。
    /// `true` かつ `cache.thumbnail_dir` が設定されている場合に生成する。
    pub thumbnail: bool,
}

impl Default for ImageCacheConfig {
    fn default() -> Self {
        Self {
            cache: CacheConfig::default(),
            thumbnail: false,
        }
    }
}

// ---------------------------------------------------------------------------
// ImageCacheWriter
// ---------------------------------------------------------------------------

/// 画像ファイル専用の更新ハンドル。
///
/// - サムネイルを `image` クレートで自動生成する (224×224 JPEG)
/// - ファイルパスは内部で `canonicalize()` を施して保存する
/// - `Clone` は低コスト (`Arc` のカウントアップのみ)
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

    /// 画像ファイルのキャッシュを登録 / 更新する。
    ///
    /// `config.thumbnail = true` かつ `config.cache.thumbnail_dir` が設定されている場合、
    /// サムネイルを自動生成して保存する。
    pub fn upsert(&self, req: UpsertImageRequest) -> Result<()> {
        let canonical = canonicalize(&req.file_path)?;
        let file_id = self.writer.upsert_file(&canonical)?;
        let conn = self.writer.write_conn()?;

        // サムネイル生成
        if self.config.thumbnail {
            if let Some(ref thumb_dir) = self.config.cache.thumbnail_dir {
                let dest = thumbnail_dest(thumb_dir, file_id);
                if !dest.exists() {
                    generate_image_thumbnail(std::path::Path::new(&canonical), &dest)?;
                }
                let thumb_canonical = canonicalize(dest.to_str().unwrap_or(""))?;
                conn.execute(
                    "INSERT INTO thumbnails (file_id, thumbnail_path) VALUES (?1, ?2)
                     ON CONFLICT(file_id) DO UPDATE SET thumbnail_path = excluded.thumbnail_path",
                    params![file_id, thumb_canonical],
                )?;
            }
        }

        // 特徴量
        if let Some(ref v) = req.clip_vector {
            conn.execute(
                "INSERT INTO image_features (file_id, clip_vector) VALUES (?1, ?2)
                 ON CONFLICT(file_id) DO UPDATE SET clip_vector = excluded.clip_vector",
                params![file_id, vec_to_blob(v)],
            )?;
        }
        Ok(())
    }

    /// 画像ファイルのキャッシュを照会する。
    pub fn lookup(&self, file_path: &str) -> anyhow::Result<LookupResult<ImageCacheEntry>> {
        self.as_reader().lookup(file_path)
    }
}

// ---------------------------------------------------------------------------
// CacheWrite trait 実装
// ---------------------------------------------------------------------------

impl CacheWrite for ImageCacheWriter {
    type Reader = ImageCacheReader;
    type Config = ImageCacheConfig;

    fn as_session(config: ImageCacheConfig) -> Result<Self> {
        let inner = CacheWriter::as_session(config.cache.clone())?;
        Ok(Self::build(inner, config))
    }

    fn oneshot(location: DbLocation) -> Result<Self> {
        let inner = CacheWriter::oneshot(location, None)?;
        Ok(Self::build(inner, ImageCacheConfig::default()))
    }

    fn as_reader(&self) -> ImageCacheReader {
        ImageCacheReader {
            reader: self.writer.as_reader(),
            config: self.config.clone(),
        }
    }

    fn delete(&self, file_path: &str) -> Result<bool> {
        self.writer.delete(file_path)
    }

    fn verify_or_invalidate(&self, file_path: &str) -> Result<bool> {
        self.writer.verify_or_invalidate(file_path)
    }

    fn list_paths(&self) -> Result<Vec<String>> {
        self.writer.list_paths()
    }
}

// ---------------------------------------------------------------------------
// ImageCacheReader
// ---------------------------------------------------------------------------

/// 画像ファイル専用の参照ハンドル。
///
/// `Clone` は低コスト。rayon の各タスクに自由に配布できる。
#[derive(Clone)]
pub struct ImageCacheReader {
    reader: file_feature_cache::CacheReader<MediaExtension>,
    config: Arc<ImageCacheConfig>,
}

impl ImageCacheReader {
    /// 継続使用・rayon 並列処理用セッション。
    pub fn as_session(config: ImageCacheConfig) -> Result<Self> {
        let reader = file_feature_cache::CacheReader::as_session(config.cache.clone())?;
        Ok(Self {
            reader,
            config: Arc::new(config),
        })
    }

    /// 単発・使い捨て用。
    pub fn oneshot(location: DbLocation) -> Result<Self> {
        let reader = file_feature_cache::CacheReader::oneshot(location)?;
        Ok(Self {
            reader,
            config: Arc::new(ImageCacheConfig::default()),
        })
    }

    /// 画像ファイルのキャッシュを照会する。
    pub fn lookup(&self, file_path: &str) -> anyhow::Result<LookupResult<ImageCacheEntry>> {
        // ファイル整合性チェック
        let row = {
            let conn = self.reader.store.read()?;
            conn.query_row(
                "SELECT id FROM files WHERE file_path = ?1",
                [file_path],
                |r| r.get::<_, i64>(0),
            )
            .optional()?
        };

        let file_id = match row {
            None => return Ok(LookupResult::Miss),
            Some(id) => id,
        };

        if !self.reader.check(file_path)? {
            return Ok(LookupResult::Invalidated);
        }

        let conn = self.reader.store.read()?;

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
            .map(|b| blob_to_vec(&b).map(|v| ImageFeatures { clip_vector: v }))
            .transpose()?;

        Ok(LookupResult::Hit(ImageCacheEntry {
            file_path: file_path.to_owned(),
            thumbnail_path,
            features,
        }))
    }
}

// ---------------------------------------------------------------------------
// CacheRead trait 実装
// ---------------------------------------------------------------------------

impl CacheRead for ImageCacheReader {
    fn check(&self, file_path: &str) -> Result<bool> {
        self.reader.check(file_path)
    }

    fn list_paths(&self) -> Result<Vec<String>> {
        self.reader.list_paths()
    }
}

// ---------------------------------------------------------------------------
// 内部ユーティリティ
// ---------------------------------------------------------------------------

fn canonicalize(path: &str) -> Result<String> {
    std::fs::canonicalize(path)
        .map(|p| p.to_string_lossy().into_owned())
        .map_err(|e| CacheError::Io {
            path: path.to_owned(),
            source: e,
        })
}
