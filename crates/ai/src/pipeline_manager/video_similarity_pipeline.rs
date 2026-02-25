use std::path::Path;

use candle_core::Device;

use crate::{
    config::video_similarity_config::VideoSimilarityConfig,
    pipeline::{
        encode::{
            audio::{AudioEncoder, whisper_encoder::WhisperEncoder},
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
    pub fn new(
        cfg: VideoSimilarityConfig,
        device: Device,
        // db_path: &Path,
        // whisper_model: WhisperModel,
    ) -> anyhow::Result<Self> {
        let clip_encoder = ClipEncoder::load(device.clone())?;
        let audio_encoder = WhisperEncoder::load(device.clone())?;
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

    /// 2 つの動画ファイルの類似度スコアを計算する
    pub fn compare(&self, path_a: &Path, path_b: &Path) -> anyhow::Result<VideoSimilarityResult> {
        let feat_a = self.get_or_extract(path_a)?;
        let feat_b = self.get_or_extract(path_b)?;
        self.calculator.compare(&feat_a, &feat_b)
    }

    // /// 動画の特徴量を事前にキャッシュに登録する
    // pub fn preload(&self, path: &Path) -> Result<()> {
    //     self.get_or_extract(path)?;
    //     Ok(())
    // }

    // pub fn cache_stats(&self) -> Result<()> {
    //     info!("{}", self.cache.stats()?);
    //     Ok(())
    // }

    // ── キャッシュ制御 ────────────────────────────────────────────────

    fn get_or_extract(&self, path: &Path) -> anyhow::Result<VideoFeatures> {
        // if let Some(cached) = self.cache.lookup(path)? {
        //     info!("[CACHE HIT]  {:?}", path.file_name().unwrap_or_default());
        //     return Ok(VideoFeatures {
        //         path: path.to_string_lossy().to_string(),
        //         video_embeddings: cached.video_embeddings,
        //         audio_embeddings: cached.audio_embeddings,
        //     });
        // }

        // todo: delete debugger
        println!("[CACHE MISS] {:?}", path.file_name().unwrap_or_default());
        let features = self.extract_features(path)?;

        // self.cache.store(
        //     path,
        //     &CachedFeatures {
        //         video_embeddings: features.video_embeddings.clone(),
        //         audio_embeddings: features.audio_embeddings.clone(),
        //     },
        // )?;

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
        let video_embeddings = self.clip_encoder.encode_frames(&frames)?;

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
        let audio_embeddings = self.audio_encoder.encode_segments(&views);

        // todo: delete debugger
        println!(
            "  → video_embeddings={}, audio_embeddings={}",
            video_embeddings.len(),
            audio_embeddings.len()
        );

        Ok(VideoFeatures {
            path: path.to_string_lossy().to_string(),
            video_embeddings,
            audio_embeddings,
        })
    }
}
