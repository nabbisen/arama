use std::path::Path;

use arama_cache::{LookupResult, UpsertVideoRequest, VideoCacheReader, VideoCacheWriter};
use arama_env::{cache_storage_path, cache_thumbnail_dir_path};
use arama_sidecar::media::video::video_engine::VideoEngine;
use candle_core::Device;

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
            video_features::VideoFeatures, video_similarity_calculator::VideoSimilarityCalculator,
            video_similarity_result::VideoSimilarityResult,
        },
    },
};

pub struct VideoSimilarityPipeline {
    cfg: VideoSimilarityConfig,
    extractor: VideoExtractor,
    clip_encoder: ClipEncoder,
    audio_encoder: Box<dyn AudioEncoder>,
    calculator: VideoSimilarityCalculator,
    // cache: FeatureCache,
}

impl VideoSimilarityPipeline {
    pub fn new(cfg: VideoSimilarityConfig) -> anyhow::Result<Self> {
        let device = ModelManager::device();

        let clip_encoder = ClipEncoder::load(device.clone())?;
        let audio_encoder = Wav2vec2Encoder::load(device)?;
        let calculator = VideoSimilarityCalculator::new(
            cfg.image_weight,
            cfg.audio_weight,
            cfg.cross_max_similarity_threshold,
        );
        let extractor = VideoExtractor::new(cfg.clone());

        // let cache = FeatureCache::open(db_path, &cfg)?;
        // cache.purge_stale_configs()?;
        // info!("{}", cache.stats()?);

        Ok(Self {
            cfg,
            extractor,
            clip_encoder,
            audio_encoder: Box::new(audio_encoder),
            calculator,
            // cache,
        })
    }

    // ── 公開 API ──────────────────────────────────────────────────────

    // /// 2 つの動画ファイルの類似度スコアを計算する
    // pub fn compare(&self, path_a: &Path, path_b: &Path) -> anyhow::Result<VideoSimilarityResult> {
    //     let feat_a = self.get_or_extract(path_a)?;
    //     let feat_b = self.get_or_extract(path_b)?;
    //     self.calculator.compare(&feat_a, &feat_b)
    // }

    /// 動画の特徴量を事前にキャッシュに登録する
    pub fn preload(&self, path: &Path) -> anyhow::Result<()> {
        self.get_or_extract(path)?;
        Ok(())
    }

    // pub fn cache_stats(&self) -> Result<()> {
    //     info!("{}", self.cache.stats()?);
    //     Ok(())
    // }

    // ── キャッシュ制御 ────────────────────────────────────────────────

    fn get_or_extract(&self, path: &Path) -> anyhow::Result<VideoFeatures> {
        let reader =
            VideoCacheReader::onetime(arama_cache::DbLocation::Custom(cache_storage_path()?))?;
        match reader.lookup(path)? {
            LookupResult::Hit(x) if x.features.is_some() => {
                // info!("[CACHE HIT]  {:?}", path.file_name().unwrap_or_default());
                let features = x.features.unwrap();
                return Ok(VideoFeatures {
                    path: path.to_string_lossy().to_string(),
                    video_embeddings: features.clip_vector.unwrap_or(vec![]),
                    audio_embeddings: features.wav2vec2_vector.unwrap_or(vec![]),
                });
            }
            _ => (),
        };

        // todo: delete debugger
        println!(
            "[CACHE MISS] {:?} ({:?})",
            path.file_name().unwrap_or_default(),
            path
        );

        let features = self.extract_features(path)?;

        let writer = VideoCacheWriter::onetime(
            arama_cache::DbLocation::Custom(cache_storage_path()?),
            Some(cache_thumbnail_dir_path()?),
            Some(
                VideoEngine::ffmpeg()
                    .expect("failed to get ffmpeg command")
                    .get_program()
                    .into(),
            ),
        )?;
        let request = UpsertVideoRequest {
            path: path.to_path_buf(),
            clip_vector: Some(features.video_embeddings.clone()),
            wav2vec2_vector: Some(features.audio_embeddings.clone()),
        };

        let _ = writer.upsert(request)?;

        Ok(features)
    }

    // ── 特徴量抽出 ────────────────────────────────────────────────────

    fn extract_features(&self, path: &Path) -> anyhow::Result<VideoFeatures> {
        // 1. 動画長を取得してサンプリングタイムスタンプを決定
        //    冒頭ゾーンに 50% 以上のサンプルを集中させる
        let duration = self.extractor.get_duration(path)?;
        let timestamps = self.cfg.compute_sample_timestamps(duration);

        // todo: delete debugger
        println!("{}", self.cfg.sampling_summary(duration));

        // 2. 映像フレームを個別シーク取得 → CLIP でバッチエンコード
        let frames = self.extractor.extract_video_frames(path, &timestamps)?;
        let video_raw_embeddings = self.clip_encoder.encode_frames(&frames)?;
        let video_embeddings = mean_embeddings(&video_raw_embeddings);

        // 3. 音声セグメントを個別シーク取得 → Whisper でエンコード
        //    映像と同じタイムスタンプを使うことで時間軸が対応する
        let sr = self.audio_encoder.required_sample_rate();
        let segments = self.extractor.extract_audio_segments_direct(
            path,
            &timestamps,
            self.cfg.audio_segment_duration_secs,
            sr,
        )?;
        let views: Vec<AudioSegmentView> = segments
            .iter()
            .map(|s| AudioSegmentView {
                start_secs: s.start_secs,
                sample_rate: s.sample_rate,
                samples: &s.samples,
            })
            .collect();
        let audio_raw_embeddings = self.audio_encoder.encode_segments(&views);
        let audio_embeddings = mean_embeddings(&audio_raw_embeddings);

        // // todo: delete debugger
        // println!(
        //     "  → video_embeddings={}, audio_embeddings={}",
        //     video_embeddings.len(),
        //     audio_embeddings.len()
        // );

        Ok(VideoFeatures {
            path: path.to_string_lossy().to_string(),
            video_embeddings,
            audio_embeddings,
        })
    }
}

fn mean_embeddings(frames: &Vec<Vec<f32>>) -> Vec<f32> {
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
    // ここで L2正規化 をしておくと、後のドット積がそのままコサイン類似度になります
    let norm = mean_vec.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for val in &mut mean_vec {
            *val /= norm;
        }
    }
    mean_vec
}
