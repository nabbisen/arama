//! `VideoCacheWriter` / `VideoCacheReader` — 動画専用キャッシュハンドル。

use std::path::{Path, PathBuf};
use std::sync::Arc;

use rayon::prelude::*;
use rusqlite::OptionalExtension;

use file_feature_cache::Result;
use file_feature_cache::{CacheConfig, CacheRead, CacheWrite, CacheWriter, DbLocation};

mod query;

use crate::core::codec::{blob_to_vec, vec_to_blob};
use crate::core::extension::MediaExtension;
use crate::core::thumbnail::{generate_video_thumbnail, thumbnail_dest};
use crate::types::{LookupResult, UpsertVideoRequest, VideoCacheEntry, VideoFeatures};
use query::{all, all_in_dir, all_in_dir_and_sub_dirs};

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct VideoCacheConfig {
    pub cache_config: CacheConfig,
    /// ffmpeg 実行ファイルのパス。`None` の場合はサムネイル生成をスキップする。
    pub ffmpeg_path: Option<PathBuf>,
}

impl Default for VideoCacheConfig {
    fn default() -> Self {
        Self {
            cache_config: CacheConfig::default(),
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
/// - `Clone` のコストは `Arc` のカウントアップのみ
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

    pub fn as_session(config: VideoCacheConfig) -> Result<Self> {
        let writer = CacheWriter::as_session(config.cache_config.clone())?;
        Ok(Self::build(writer, config))
    }

    pub fn onetime(
        location: DbLocation,
        thumbnail_dir: Option<PathBuf>,
        ffmpeg_path: Option<PathBuf>,
    ) -> Result<Self> {
        let writer = CacheWriter::onetime(location)?;
        let config = VideoCacheConfig {
            cache_config: CacheConfig {
                thumbnail_dir,
                ..CacheConfig::default()
            },
            ffmpeg_path,
        };
        Ok(Self::build(writer, config))
    }

    // -----------------------------------------------------------------------
    // 更新 API
    // -----------------------------------------------------------------------

    pub fn upsert(&self, req: UpsertVideoRequest) -> Result<()> {
        let id = self.writer.refresh(&req.path)?;
        self.write_features(id, &req)
    }

    /// `upsert` の一括版。各リクエストに対して `(PathBuf, Result<()>)` を返す。
    ///
    /// ## 並列化戦略
    ///
    /// - **fingerprint 計算**: rayon で並列実行
    /// - **DB 書き込み**: write pool の制約に従い直列で処理
    ///
    /// 個々のエラーは `Err` として各要素に格納され、他のリクエストの処理は継続する。
    pub fn upsert_all(&self, reqs: Vec<UpsertVideoRequest>) -> Vec<(PathBuf, Result<()>)> {
        let paths: Vec<&Path> = reqs.iter().map(|r| r.path.as_path()).collect();
        let id_results = self.writer.refresh_all(&paths);

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

    pub fn as_reader(&self) -> VideoCacheReader {
        VideoCacheReader {
            reader: self.writer.as_reader(),
            config: self.config.clone(),
        }
    }

    pub fn lookup(&self, path: &Path) -> Result<LookupResult<VideoCacheEntry>> {
        self.as_reader().lookup(path)
    }

    // -----------------------------------------------------------------------
    // 内部
    // -----------------------------------------------------------------------

    fn write_features(&self, id: i64, req: &UpsertVideoRequest) -> Result<()> {
        let conn = self.writer.write_conn()?;

        if let (Some(ffmpeg), Some(thumb_dir)) = (
            &self.config.ffmpeg_path,
            &self.config.cache_config.thumbnail_dir,
        ) {
            let dest = thumbnail_dest(thumb_dir, id);
            if !dest.exists() {
                generate_video_thumbnail(&req.path, &dest, ffmpeg)?;
            }
            conn.execute(
                "INSERT INTO thumbnails (id, thumbnail_path) VALUES (?1, ?2)
                 ON CONFLICT(id) DO UPDATE SET thumbnail_path = excluded.thumbnail_path",
                rusqlite::params![id, dest.to_string_lossy().as_ref()],
            )?;
        }

        let clip_blob = req.clip_vector.as_deref().map(vec_to_blob);
        let wav_blob = req.wav2vec2_vector.as_deref().map(vec_to_blob);

        if clip_blob.is_some() || wav_blob.is_some() {
            conn.execute(
                "INSERT INTO video_features (id, clip_vector, wav2vec2_vector)
                 VALUES (?1, ?2, ?3)
                 ON CONFLICT(id) DO UPDATE
                     SET clip_vector     = COALESCE(excluded.clip_vector,     clip_vector),
                         wav2vec2_vector = COALESCE(excluded.wav2vec2_vector, wav2vec2_vector)",
                rusqlite::params![id, clip_blob, wav_blob],
            )?;
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// CacheWrite 実装
// ---------------------------------------------------------------------------

impl CacheWrite for VideoCacheWriter {
    type Reader = VideoCacheReader;

    fn as_session(cache_config: CacheConfig) -> Result<Self> {
        VideoCacheWriter::as_session(VideoCacheConfig {
            cache_config,
            ffmpeg_path: None,
        })
    }

    fn onetime(location: DbLocation) -> Result<Self> {
        VideoCacheWriter::onetime(location, None, None)
    }

    fn as_reader(&self) -> VideoCacheReader {
        VideoCacheWriter::as_reader(self)
    }

    fn refresh(&self, path: &Path) -> Result<i64> {
        self.writer.refresh(path)
    }

    fn refresh_all(&self, paths: &[&Path]) -> Vec<(PathBuf, Result<i64>)> {
        self.writer.refresh_all(paths)
    }

    fn delete(&self, path: &Path) -> Result<bool> {
        VideoCacheWriter::delete(self, path)
    }

    fn list_paths(&self) -> Result<Vec<String>> {
        VideoCacheWriter::list_paths(self)
    }
}

// ---------------------------------------------------------------------------
// VideoCacheReader
// ---------------------------------------------------------------------------

/// 動画ファイル専用の参照ハンドル。`Clone` のコストは `Arc` のカウントアップのみ。
#[derive(Clone)]
pub struct VideoCacheReader {
    reader: file_feature_cache::CacheReader<MediaExtension>,
    config: Arc<VideoCacheConfig>,
}

impl VideoCacheReader {
    pub fn as_session(config: VideoCacheConfig) -> Result<Self> {
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
            config: Arc::new(VideoCacheConfig::default()),
        })
    }

    pub fn lookup(&self, path: &Path) -> Result<LookupResult<VideoCacheEntry>> {
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
                "SELECT clip_vector, wav2vec2_vector FROM video_features WHERE id = ?1",
                [id],
                |r| {
                    Ok((
                        r.get::<_, Option<Vec<u8>>>(0)?,
                        r.get::<_, Option<Vec<u8>>>(1)?,
                    ))
                },
            )
            .optional()?
            .map(|(clip_raw, wav_raw)| -> Result<VideoFeatures> {
                Ok(VideoFeatures {
                    clip_vector: clip_raw.map(|b| blob_to_vec(&b)).transpose()?,
                    wav2vec2_vector: wav_raw.map(|b| blob_to_vec(&b)).transpose()?,
                })
            })
            .transpose()?;

        Ok(LookupResult::Hit(VideoCacheEntry {
            path: canonical,
            thumbnail_path,
            features,
        }))
    }

    /// `lookup` の一括版。read pool の複数コネクションを使って rayon で並列実行する。
    pub fn lookup_all(
        &self,
        paths: &[&Path],
    ) -> Vec<(PathBuf, Result<LookupResult<VideoCacheEntry>>)> {
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

    pub fn all(&self) -> Result<Vec<Result<VideoCacheEntry>>> {
        let conn = self.reader.read_conn()?;
        let ret = all(&conn);
        match ret {
            Ok(x) => Ok(x
                .into_iter()
                .map(|x| match x {
                    Ok(x) => {
                        let features = if x.image_features.is_some() || x.audio_features.is_some() {
                            Some(VideoFeatures {
                                clip_vector: x.image_features,
                                wav2vec2_vector: x.audio_features,
                            })
                        } else {
                            None
                        };

                        Ok(VideoCacheEntry {
                            path: x.path,
                            thumbnail_path: x.thumbnail_path,
                            features,
                        })
                    }
                    Err(err) => Err(err),
                })
                .collect::<Vec<_>>()),
            Err(err) => Err(err),
        }
    }

    pub fn all_in_dir(&self, path: &Path) -> Result<Vec<Result<VideoCacheEntry>>> {
        let conn = self.reader.read_conn()?;
        let ret = all_in_dir(path, &conn);
        match ret {
            Ok(x) => Ok(x
                .into_iter()
                .map(|x| match x {
                    Ok(x) => {
                        let features = if x.image_features.is_some() || x.audio_features.is_some() {
                            Some(VideoFeatures {
                                clip_vector: x.image_features,
                                wav2vec2_vector: x.audio_features,
                            })
                        } else {
                            None
                        };

                        Ok(VideoCacheEntry {
                            path: x.path,
                            thumbnail_path: x.thumbnail_path,
                            features,
                        })
                    }
                    Err(err) => Err(err),
                })
                .collect::<Vec<_>>()),
            Err(err) => Err(err),
        }
    }

    pub fn all_in_dir_and_sub_dirs(&self, path: &Path) -> Result<Vec<Result<VideoCacheEntry>>> {
        let conn = self.reader.read_conn()?;
        let ret = all_in_dir_and_sub_dirs(path, &conn);
        match ret {
            Ok(x) => Ok(x
                .into_iter()
                .map(|x| match x {
                    Ok(x) => {
                        let features = if x.image_features.is_some() || x.audio_features.is_some() {
                            Some(VideoFeatures {
                                clip_vector: x.image_features,
                                wav2vec2_vector: x.audio_features,
                            })
                        } else {
                            None
                        };

                        Ok(VideoCacheEntry {
                            path: x.path,
                            thumbnail_path: x.thumbnail_path,
                            features,
                        })
                    }
                    Err(err) => Err(err),
                })
                .collect::<Vec<_>>()),
            Err(err) => Err(err),
        }
    }
}

// ---------------------------------------------------------------------------
// CacheRead 実装
// ---------------------------------------------------------------------------

impl CacheRead for VideoCacheReader {
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
