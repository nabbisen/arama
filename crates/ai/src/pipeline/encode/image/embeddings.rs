use std::path::{Path, PathBuf};

use anyhow::anyhow;
use arama_cache::{
    CacheConfig, DbLocation, ImageCacheConfig, ImageCacheWriter, LookupResult, UpsertImageRequest,
};
use arama_env::{VIDEO_EXTENSION_ALLOWLIST, cache_storage_path, cache_thumbnail_dir_path};

use crate::{
    config::video_similarity_config::VideoSimilarityConfig,
    pipeline::encode::image::{clip, clip_calculator},
    pipeline_manager::video_similarity_pipeline::{VideoPreloadOutcome, VideoSimilarityPipeline},
};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EmbeddingRunReport {
    pub processed: usize,
    pub skipped: Vec<EmbeddingFileIssue>,
    pub cache_write_failures: Vec<EmbeddingFileIssue>,
}

impl EmbeddingRunReport {
    pub fn has_warnings(&self) -> bool {
        !self.skipped.is_empty() || !self.cache_write_failures.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmbeddingFileIssue {
    pub path: PathBuf,
    pub message: String,
}

pub async fn image_embedding(paths: Vec<PathBuf>) -> anyhow::Result<EmbeddingRunReport> {
    let has_video = paths.iter().any(|path| is_video_path(path));
    let has_image = paths.iter().any(|path| !is_video_path(path));

    let db_location = DbLocation::Custom(cache_storage_path()?);
    let cache_writer = ImageCacheWriter::as_session(ImageCacheConfig {
        cache_config: CacheConfig {
            db_location,
            read_conns: 4,
            thumbnail_dir: Some(cache_thumbnail_dir_path()?),
        },
    })?;

    let calculator = match clip_calculator() {
        Ok(calculator) => Some(calculator),
        Err(err) if has_image && !has_video => {
            return Err(anyhow!("failed to load clip calculator: {}", err));
        }
        Err(_) => None,
    };

    let video_similarity_config = VideoSimilarityConfig::default();
    let pipeline = has_video.then(|| VideoSimilarityPipeline::new(video_similarity_config));
    if !has_usable_requested_modality(
        has_image,
        calculator.is_some(),
        has_video,
        pipeline
            .as_ref()
            .is_some_and(VideoSimilarityPipeline::has_any_modality),
    ) {
        return Err(anyhow!(
            "failed to initialize any requested embedding modality"
        ));
    }

    let mut report = EmbeddingRunReport::default();

    for path in paths {
        // Yield to the async runtime at each file boundary so that
        // Task::abortable() can cancel this loop when the user switches
        // to a different directory before indexing finishes.
        tokio::task::yield_now().await;

        if is_video_path(&path) {
            let Some(pipeline) = &pipeline else {
                report.skipped.push(EmbeddingFileIssue {
                    path,
                    message: "video pipeline unavailable".to_owned(),
                });
                continue;
            };
            let outcome = pipeline.preload(&path);
            record_video_outcome(&mut report, path, outcome);
            continue;
        }

        let Some(calculator) = &calculator else {
            report.skipped.push(EmbeddingFileIssue {
                path,
                message: "image embedding modality unavailable".to_owned(),
            });
            continue;
        };

        match cache_writer.as_reader().lookup(&path) {
            Ok(LookupResult::Hit(x)) if x.features.is_some() => {
                report.processed += 1;
                continue;
            }
            Ok(_) => (),
            Err(err) => {
                report.skipped.push(EmbeddingFileIssue {
                    path,
                    message: err.to_string(),
                });
                continue;
            }
        }

        let embedding = match clip(&path, calculator) {
            Ok(x) => x,
            Err(err) => {
                report.skipped.push(EmbeddingFileIssue {
                    path,
                    message: format!("failed to clip calculation: {err}"),
                });
                continue;
            }
        };
        let req = UpsertImageRequest {
            path: path.clone(),
            // thumbnail_path: path,
            clip_vector: Some(embedding.embedding),
        };
        match cache_writer.upsert(req) {
            Ok(_) => report.processed += 1,
            Err(err) => {
                report.processed += 1;
                report.cache_write_failures.push(EmbeddingFileIssue {
                    path,
                    message: format!("failed to set embedding: {err}"),
                });
            }
        }
    }

    Ok(report)
}

fn is_video_path(path: &Path) -> bool {
    path.extension().is_some_and(|x| {
        VIDEO_EXTENSION_ALLOWLIST.contains(&x.to_string_lossy().to_string().as_str())
    })
}

fn has_usable_requested_modality(
    has_image: bool,
    has_image_calculator: bool,
    has_video: bool,
    has_video_modality: bool,
) -> bool {
    (has_image && has_image_calculator) || (has_video && has_video_modality)
}

fn record_video_outcome(
    report: &mut EmbeddingRunReport,
    path: PathBuf,
    outcome: VideoPreloadOutcome,
) {
    match outcome {
        VideoPreloadOutcome::Processed => report.processed += 1,
        VideoPreloadOutcome::Skipped(message) => {
            report.skipped.push(EmbeddingFileIssue { path, message });
        }
        VideoPreloadOutcome::CacheWriteFailed(message) => {
            report.processed += 1;
            report
                .cache_write_failures
                .push(EmbeddingFileIssue { path, message });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_counts_cache_write_failure_as_processed_warning() {
        let mut report = EmbeddingRunReport::default();

        record_video_outcome(
            &mut report,
            PathBuf::from("video.mp4"),
            VideoPreloadOutcome::CacheWriteFailed("disk full".to_owned()),
        );

        assert_eq!(report.processed, 1);
        assert!(report.skipped.is_empty());
        assert_eq!(report.cache_write_failures.len(), 1);
        assert!(report.has_warnings());
    }

    #[test]
    fn report_counts_skipped_file_without_aborting_later_processed_files() {
        let mut report = EmbeddingRunReport::default();

        record_video_outcome(
            &mut report,
            PathBuf::from("bad.mp4"),
            VideoPreloadOutcome::Skipped("decode failed".to_owned()),
        );
        record_video_outcome(
            &mut report,
            PathBuf::from("good.mp4"),
            VideoPreloadOutcome::Processed,
        );

        assert_eq!(report.processed, 1);
        assert_eq!(report.skipped.len(), 1);
        assert!(report.cache_write_failures.is_empty());
        assert!(report.has_warnings());
    }

    #[test]
    fn no_requested_modality_is_usable_for_mixed_selection_is_fatal() {
        assert!(!has_usable_requested_modality(true, false, true, false));
    }
}
