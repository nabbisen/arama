use std::path::Path;

use crate::{
    config::video_similarity_config::VideoSimilarityConfig,
    model::model_manager::ModelManager,
    pipeline::{
        encode::{
            audio::{AudioEncoder, wav2vec2_encoder::Wav2vec2Encoder},
            image::clip_encoder::ClipEncoder,
        },
        extract::video_extractor::{VideoExtractor, audio_segment::AudioSegmentView},
        score::similarity::video::{
            video_features::VideoFeatures, video_similarity_calculator::has_signal,
        },
    },
};
use arama_cache::{LookupResult, UpsertVideoRequest, VideoCacheReader, VideoCacheWriter};
use arama_env::{cache_storage_path, cache_thumbnail_dir_path};
use arama_sidecar::media::video::video_engine::FfmpegToolchain;

pub struct VideoSimilarityPipeline {
    cfg: VideoSimilarityConfig,
    extractor: VideoExtractor,
    clip_encoder: Option<ClipEncoder>,
    clip_setup_error: Option<String>,
    audio_encoder: Option<Box<dyn AudioEncoder>>,
    audio_setup_error: Option<String>,
    // cache: FeatureCache,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VideoPreloadOutcome {
    Processed,
    Skipped(String),
    CacheWriteFailed(String),
}

impl VideoSimilarityPipeline {
    pub fn new(cfg: VideoSimilarityConfig, toolchain: FfmpegToolchain) -> Self {
        let device = ModelManager::device();

        let (clip_encoder, clip_setup_error) = match ClipEncoder::load(device.clone()) {
            Ok(encoder) => (Some(encoder), None),
            Err(err) => (None, Some(err.to_string())),
        };
        let (audio_encoder, audio_setup_error) = match Wav2vec2Encoder::load(device) {
            Ok(encoder) => (Some(Box::new(encoder) as Box<dyn AudioEncoder>), None),
            Err(err) => (None, Some(err.to_string())),
        };
        let extractor = VideoExtractor::new(cfg.clone(), toolchain);

        // let cache = FeatureCache::open(db_path, &cfg)?;
        // cache.purge_stale_configs()?;
        // info!("{}", cache.stats()?);

        Self {
            cfg,
            extractor,
            clip_encoder,
            clip_setup_error,
            audio_encoder,
            audio_setup_error,
            // cache,
        }
    }

    // Public API.

    // /// Calculate a similarity score for two video files.
    // pub fn compare(&self, path_a: &Path, path_b: &Path) -> anyhow::Result<VideoSimilarityResult> {
    //     let feat_a = self.get_or_extract(path_a)?;
    //     let feat_b = self.get_or_extract(path_b)?;
    //     self.calculator.compare(&feat_a, &feat_b)
    // }

    /// Preload video features into the cache.
    pub fn preload(&self, path: &Path) -> VideoPreloadOutcome {
        self.get_or_extract(path)
    }

    pub fn has_any_modality(&self) -> bool {
        self.clip_encoder.is_some() || self.audio_encoder.is_some()
    }

    // pub fn cache_stats(&self) -> Result<()> {
    //     info!("{}", self.cache.stats()?);
    //     Ok(())
    // }

    // Cache control.

    fn get_or_extract(&self, path: &Path) -> VideoPreloadOutcome {
        let cache_path = match cache_storage_path() {
            Ok(path) => path,
            Err(err) => return VideoPreloadOutcome::Skipped(err.to_string()),
        };
        let reader =
            match VideoCacheReader::onetime(arama_cache::DbLocation::Custom(cache_path.clone())) {
                Ok(reader) => reader,
                Err(err) => return VideoPreloadOutcome::Skipped(err.to_string()),
            };
        match reader.lookup(path) {
            Ok(LookupResult::Hit(x)) if usable_cached_features(&x.features) => {
                // info!("[CACHE HIT]  {:?}", path.file_name().unwrap_or_default());
                return VideoPreloadOutcome::Processed;
            }
            Err(err) => return VideoPreloadOutcome::Skipped(err.to_string()),
            _ => (),
        };

        let features = match self.extract_features(path) {
            Ok(features) => features,
            Err(err) => return VideoPreloadOutcome::Skipped(err),
        };

        let ffmpeg_path = Some(self.extractor.ffmpeg_path().to_path_buf());
        let thumbnail_dir = cache_thumbnail_dir_path().ok();
        let writer = VideoCacheWriter::onetime(
            arama_cache::DbLocation::Custom(cache_path),
            thumbnail_dir,
            ffmpeg_path,
        );
        let writer = match writer {
            Ok(writer) => writer,
            Err(err) => return VideoPreloadOutcome::CacheWriteFailed(err.to_string()),
        };
        let request = UpsertVideoRequest {
            path: path.to_path_buf(),
            clip_vector: valid_feature_vector(features.video_embeddings.clone()),
            wav2vec2_vector: valid_feature_vector(features.audio_embeddings.clone()),
        };

        if let Err(err) = writer.upsert(request) {
            return VideoPreloadOutcome::CacheWriteFailed(err.to_string());
        }

        VideoPreloadOutcome::Processed
    }

    // Feature extraction.

    fn extract_features(&self, path: &Path) -> Result<VideoFeatures, String> {
        // 1. Read video duration and choose sampling timestamps.
        //    More than half of the samples are concentrated in the head zone.
        let duration = self
            .extractor
            .get_duration(path)
            .map_err(|err| format!("failed to read video duration: {err}"))?;
        let timestamps = self.cfg.compute_sample_timestamps(duration);

        // 2. Seek video frames individually, then batch-encode them with CLIP.
        let (video_embeddings, frame_failures) = match &self.clip_encoder {
            Some(encoder) => {
                let frames = self
                    .extractor
                    .extract_video_frames_report(path, &timestamps);
                let extraction_failures = frames.failures;
                if frames.frames.is_empty() {
                    (vec![], extraction_failures)
                } else {
                    let frame_count = frames.frames.len();
                    let raw_embeddings = encoder.encode_frames(&frames.frames).unwrap_or_default();
                    // Task 040 (audit A4): `frames_to_tensor` excludes any
                    // frame it rejects as malformed rather than trusting
                    // it (see `build_frame_batch`'s own doc comment) - the
                    // gap between frames sent in and vectors returned is
                    // exactly that count, mirroring Task 042's audio-side
                    // fix for the same shape of defect.
                    let validation_failures = frame_count.saturating_sub(raw_embeddings.len());
                    (
                        mean_embeddings(&raw_embeddings),
                        extraction_failures + validation_failures,
                    )
                }
            }
            None => (vec![], 0),
        };

        // 3. Seek audio segments individually, then encode them with wav2vec2.
        //    Using the same timestamps as video keeps the timelines aligned.
        let (audio_embeddings, audio_failures) = match &self.audio_encoder {
            Some(encoder) => {
                let sr = encoder.required_sample_rate();
                let segments = self.extractor.extract_audio_segments_direct_report(
                    path,
                    &timestamps,
                    self.cfg.audio_segment_duration_secs,
                    sr,
                );
                let extraction_failures = segments.failures;
                let views: Vec<AudioSegmentView> = segments
                    .segments
                    .iter()
                    .map(|s| AudioSegmentView {
                        start_secs: s.start_secs,
                        sample_rate: s.sample_rate,
                        samples: &s.samples,
                    })
                    .collect();
                let segment_count = views.len();
                let audio_raw_embeddings = encoder.encode_segments(&views);
                // Task 042 (audit A10): `encode_segments` excludes any
                // segment it could not encode rather than returning a
                // zero vector for it (see the trait's own doc comment) -
                // the gap between segments sent in and vectors returned
                // is exactly that count, and it did not exist before this
                // fix: an encode failure that was not also an
                // *extraction* failure was counted nowhere.
                let encode_failures = segment_count.saturating_sub(audio_raw_embeddings.len());
                (
                    mean_embeddings(&audio_raw_embeddings),
                    extraction_failures + encode_failures,
                )
            }
            None => (vec![], 0),
        };

        if valid_feature_vector(video_embeddings.clone()).is_none()
            && valid_feature_vector(audio_embeddings.clone()).is_none()
        {
            return Err(self.unavailable_modalities_message(frame_failures, audio_failures));
        }

        Ok(VideoFeatures {
            path: path.to_string_lossy().to_string(),
            video_embeddings,
            audio_embeddings,
        })
    }

    fn unavailable_modalities_message(
        &self,
        frame_failures: usize,
        audio_failures: usize,
    ) -> String {
        let mut reasons = Vec::new();
        if let Some(err) = &self.clip_setup_error {
            reasons.push(format!("frame modality unavailable: {err}"));
        } else if frame_failures > 0 {
            // Task 040 (audit A4): "processing", not "extraction" - this
            // count is now extraction failures plus validation failures
            // folded together (see extract_features's own comment), so
            // "extraction" alone would be inaccurate for the latter. Same
            // correction as Task 042 made on the audio side.
            reasons.push(format!(
                "frame processing failed at {frame_failures} sample points"
            ));
        }
        if let Some(err) = &self.audio_setup_error {
            reasons.push(format!("audio modality unavailable: {err}"));
        } else if audio_failures > 0 {
            // Task 042: "processing", not "extraction" - this count is
            // now extraction failures plus encode failures folded
            // together (see extract_features's own comment), so
            // "extraction" alone would be inaccurate for the latter.
            reasons.push(format!(
                "audio processing failed at {audio_failures} sample points"
            ));
        }
        if reasons.is_empty() {
            "no usable video or audio embeddings extracted".to_owned()
        } else {
            reasons.join("; ")
        }
    }
}

fn usable_cached_features(features: &Option<arama_cache::VideoFeatures>) -> bool {
    features.as_ref().is_some_and(|features| {
        features.clip_vector.as_deref().is_some_and(has_signal)
            || features.wav2vec2_vector.as_deref().is_some_and(has_signal)
    })
}

fn valid_feature_vector(vector: Vec<f32>) -> Option<Vec<f32>> {
    has_signal(&vector).then_some(vector)
}

fn mean_embeddings(frames: &[Vec<f32>]) -> Vec<f32> {
    if frames.is_empty() {
        return vec![];
    }
    let dim = frames[0].len();
    let mut mean_vec = vec![0.0; dim];
    for frame in frames {
        for (i, val) in frame.iter().enumerate() {
            mean_vec[i] += val;
        }
    }
    let f_n = frames.len() as f32;
    for val in &mut mean_vec {
        *val /= f_n;
    }
    // L2-normalize here so later dot products are cosine similarity.
    let norm = mean_vec.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for val in &mut mean_vec {
            *val /= norm;
        }
    }
    mean_vec
}
