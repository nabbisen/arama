//! `VideoCacheWriter` / `VideoCacheReader` — 動画専用キャッシュハンドル。

use std::path::PathBuf;
use std::sync::Arc;

use arama_repr::codec::{blob_to_vec, vec_to_blob};
use file_feature_cache::{CacheConfig, CacheError, CacheRead, CacheWrite, CacheWriter, DbLocation};
use rusqlite::{OptionalExtension, params};

use crate::core::extension::MediaExtension;
use crate::core::thumbnail::{generate_video_thumbnail, thumbnail_dest};
use crate::types::{LookupResult, UpsertVideoRequest, VideoCacheEntry, VideoFeatures};

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

/// [`VideoCacheWriter::as_session`] に渡す設定。
#[derive(Debug, Clone)]
pub struct VideoCacheConfig {
    /// エンジン設定 (DB パス・コネクション数・サムネイルディレクトリ)。
    pub cache: CacheConfig,
    /// サムネイルを自動生成するか。
    /// `true` かつ `cache.thumbnail_dir` と `ffmpeg_path` が設定されている場合に生成する。
    pub thumbnail: bool,
    /// ffmpeg 実行ファイルのパス。`None` の場合はサムネイル生成をスキップする。
    pub ffmpeg_path: Option<PathBuf>,
}

impl Default for VideoCacheConfig {
    fn default() -> Self {
        Self {
            cache: CacheConfig::default(),
            thumbnail: false,
            ffmpeg_path: None,
        }
    }
}

// ---------------------------------------------------------------------------
// VideoCacheWriter
// ---------------------------------------------------------------------------

/// 動画ファイル専用の更新ハンドル。
///
/// - サムネイルを ffmpeg で自動生成する (5 秒時点、失敗時は 0 秒にフォールバック)
/// - ファイルパスは内部で `canonicalize()` を施して保存する
/// - `Clone` は低コスト (`Arc` のカウントアップのみ)
#[derive(Clone)]
pub struct VideoCacheWriter {
    writer: CacheWriter<MediaExtension>,
    config: Arc<VideoCacheConfig>,
}

impl VideoCacheWriter {
    fn build(writer: CacheWriter<MediaExtension>, config: VideoCacheConfig) -> Self {
        Self {
            writer,
            config: Arc::new(config),
        }
    }

    /// 動画ファイルのキャッシュを登録 / 更新する。
    ///
    /// `config.thumbnail = true` かつ `config.ffmpeg_path` と `config.cache.thumbnail_dir`
    /// が設定されている場合、サムネイルを自動生成して保存する。
    pub fn upsert(&self, req: UpsertVideoRequest) -> Result<(), CacheError> {
        let canonical = canonicalize(&req.file_path)?;
        let file_id = self.writer.upsert_file(&canonical)?;
        let conn = self.writer.write_conn()?;

        // サムネイル生成
        if self.config.thumbnail {
            if let (Some(ffmpeg), Some(thumb_dir)) =
                (&self.config.ffmpeg_path, &self.config.cache.thumbnail_dir)
            {
                let dest = thumbnail_dest(thumb_dir, file_id);
                if !dest.exists() {
                    generate_video_thumbnail(std::path::Path::new(&canonical), &dest, ffmpeg)?;
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
        match (req.clip_vector.as_ref(), req.wav2vec2_vector.as_ref()) {
            (Some(c), Some(w)) => {
                conn.execute(
                    "INSERT INTO video_features (file_id, clip_vector, wav2vec2_vector)
                     VALUES (?1, ?2, ?3)
                     ON CONFLICT(file_id) DO UPDATE
                         SET clip_vector     = excluded.clip_vector,
                             wav2vec2_vector = excluded.wav2vec2_vector",
                    params![file_id, vec_to_blob(c), vec_to_blob(w)],
                )?;
            }
            (Some(c), None) => {
                conn.execute(
                    "UPDATE video_features SET clip_vector = ?2 WHERE file_id = ?1",
                    params![file_id, vec_to_blob(c)],
                )?;
            }
            (None, Some(w)) => {
                conn.execute(
                    "UPDATE video_features SET wav2vec2_vector = ?2 WHERE file_id = ?1",
                    params![file_id, vec_to_blob(w)],
                )?;
            }
            (None, None) => {}
        }
        Ok(())
    }

    /// 動画ファイルのキャッシュを照会する。
    pub fn lookup(&self, file_path: &str) -> anyhow::Result<LookupResult<VideoCacheEntry>> {
        self.as_reader().lookup(file_path)
    }
}

// ---------------------------------------------------------------------------
// CacheWrite trait 実装
// ---------------------------------------------------------------------------

impl CacheWrite for VideoCacheWriter {
    type Reader = VideoCacheReader;
    type Config = VideoCacheConfig;

    fn as_session(config: VideoCacheConfig) -> Result<Self, CacheError> {
        let inner = CacheWriter::as_session(config.cache.clone())?;
        Ok(Self::build(inner, config))
    }

    fn oneshot(location: DbLocation) -> Result<Self, CacheError> {
        let writer = CacheWriter::oneshot(location, None)?;
        Ok(Self::build(writer, VideoCacheConfig::default()))
    }

    fn as_reader(&self) -> VideoCacheReader {
        VideoCacheReader {
            reader: self.writer.as_reader(),
            config: self.config.clone(),
        }
    }

    fn delete(&self, file_path: &str) -> Result<bool, CacheError> {
        self.writer.delete(file_path)
    }

    fn verify_or_invalidate(&self, file_path: &str) -> Result<bool, CacheError> {
        self.writer.verify_or_invalidate(file_path)
    }

    fn list_paths(&self) -> Result<Vec<String>, CacheError> {
        self.writer.list_paths()
    }
}

// ---------------------------------------------------------------------------
// VideoCacheReader
// ---------------------------------------------------------------------------

/// 動画ファイル専用の参照ハンドル。
///
/// `Clone` は低コスト。rayon の各タスクに自由に配布できる。
#[derive(Clone)]
pub struct VideoCacheReader {
    reader: file_feature_cache::CacheReader<MediaExtension>,
    config: Arc<VideoCacheConfig>,
}

impl VideoCacheReader {
    /// 継続使用・rayon 並列処理用セッション。
    pub fn as_session(config: VideoCacheConfig) -> anyhow::Result<Self> {
        let reader = file_feature_cache::CacheReader::as_session(config.cache.clone())?;
        Ok(Self {
            reader,
            config: Arc::new(config),
        })
    }

    /// 単発・使い捨て用。
    pub fn oneshot(location: DbLocation) -> anyhow::Result<Self> {
        let reader = file_feature_cache::CacheReader::oneshot(location)?;
        Ok(Self {
            reader,
            config: Arc::new(VideoCacheConfig::default()),
        })
    }

    /// 動画ファイルのキャッシュを照会する。
    pub fn lookup(&self, file_path: &str) -> anyhow::Result<LookupResult<VideoCacheEntry>> {
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
                "SELECT clip_vector, wav2vec2_vector FROM video_features WHERE file_id = ?1",
                [file_id],
                |r| Ok((r.get::<_, Vec<u8>>(0)?, r.get::<_, Vec<u8>>(1)?)),
            )
            .optional()?
            .map(|(clip, wav)| -> anyhow::Result<VideoFeatures> {
                Ok(VideoFeatures {
                    clip_vector: blob_to_vec(&clip)?,
                    wav2vec2_vector: blob_to_vec(&wav)?,
                })
            })
            .transpose()?;

        Ok(LookupResult::Hit(VideoCacheEntry {
            file_path: file_path.to_owned(),
            thumbnail_path,
            features,
        }))
    }
}

// ---------------------------------------------------------------------------
// CacheRead trait 実装
// ---------------------------------------------------------------------------

impl CacheRead for VideoCacheReader {
    fn check(&self, file_path: &str) -> Result<bool, CacheError> {
        self.reader.check(file_path)
    }

    fn list_paths(&self) -> Result<Vec<String>, CacheError> {
        self.reader.list_paths()
    }
}

// ---------------------------------------------------------------------------
// 内部ユーティリティ
// ---------------------------------------------------------------------------

fn canonicalize(path: &str) -> Result<String, CacheError> {
    std::fs::canonicalize(path)
        .map(|p| p.to_string_lossy().into_owned())
        .map_err(|e| CacheError::Io {
            path: path.to_owned(),
            source: e,
        })
}
