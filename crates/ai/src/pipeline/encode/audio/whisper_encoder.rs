//! Whisper Audio Encoder
//!
//! Whisper の Encoder 部分のみを使用して音声セグメントを
//! 固定長の埋め込みベクトルに変換する。
//! Decoder（テキスト生成）は一切使用しない。
//!
//! モデル構造:
//!   Audio → log-mel spectrogram [1, 80, 3000]
//!         → Conv1d × 2 + positional embedding
//!         → Transformer Encoder blocks
//!         → [1, 1500, hidden_dim]
//!         → mean pooling（時間軸）
//!         → L2 正規化
//!         → [hidden_dim] 埋め込みベクトル
//!
//! hidden_dim: tiny=384 / base=512 / small=768 / medium=1024

use anyhow::Context;
use candle_core::{DType, Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::whisper::{Config, audio::pcm_to_mel, model::Whisper};

use crate::{
    model::model_container::wav2vec2::{self, HIDDEN_DIM},
    pipeline::{
        encode::audio::{AudioEncoder, mel_filters::build_mel_filterbank},
        extract::video_extractor::audio_segment::AudioSegmentView,
    },
};

// ─── Whisper 前処理定数 ────────────────────────────────────────────────

pub const WHISPER_SAMPLE_RATE: u32 = 16_000;

const N_MELS: usize = 80;
const FFT_SIZE: usize = 400; // 25ms @ 16kHz
// const HOP_SIZE: usize = 160; // 10ms @ 16kHz
// const CHUNK_SECS: f64 = 30.0;
const CHUNK_FRAMES: usize = 3_000; // 30s × 100fps
const F_MIN: f32 = 0.0;
const F_MAX: f32 = 8_000.0;

pub struct WhisperEncoder {
    model: Whisper,
    config: Config,
    mel_filters: Vec<f32>,
    device: Device,
    hidden_dim: usize,
}

impl WhisperEncoder {
    /// HuggingFace Hub からモデルをダウンロードしてロードする
    // pub async fn load(kind: WhisperModel, device: Device) -> Result<Self> {
    pub fn load(device: Device) -> anyhow::Result<Self> {
        // let hf_id = kind.hf_id();
        // let hidden_dim = kind.hidden_dim();

        // info!(
        //     "Loading Whisper encoder: {} (hidden_dim={})",
        //     hf_id, hidden_dim
        // );

        // let api = Api::new()?;
        // let repo = api.repo(Repo::new(hf_id.into(), RepoType::Model));

        // let model_safetensors_file = repo.get("model.safetensors").await?;
        // let config_json_file = repo.get("config.json").await?;
        let model: crate::model::model_container::ModelContainer = wav2vec2::model();
        let model_safetensors_path = model.safetensors_path()?;
        let config_json_path = model.config_json_path()?;

        let cfg_str = std::fs::read_to_string(&config_json_path)?;
        let config: Config =
            serde_json::from_str(&cfg_str).context("Failed to parse Whisper config.json")?;

        let vb = unsafe {
            VarBuilder::from_mmaped_safetensors(&[model_safetensors_path], DType::F32, &device)?
        };

        let model = Whisper::load(&vb, config.clone()).context("Failed to load Whisper model")?;

        // todo: delete debugger
        println!("Whisper encoder ready on {:?}", device);

        Ok(Self {
            model,
            config,
            mel_filters: build_mel_filterbank(WHISPER_SAMPLE_RATE, FFT_SIZE, N_MELS, F_MIN, F_MAX),
            device,
            hidden_dim: HIDDEN_DIM,
        })
    }

    /// 1 セグメントを Encoder に通してベクトルを生成する
    fn encode_one(&self, seg: &AudioSegmentView) -> anyhow::Result<Vec<f32>> {
        let mel_vec = pcm_to_mel(&self.config, seg.samples, &self.mel_filters);
        let mel = Tensor::from_vec(mel_vec, (1, N_MELS, CHUNK_FRAMES), &self.device)?;

        let hidden = self.model.encoder.clone().forward(&mel, true)?;
        // hidden: [1, 1500, hidden_dim]

        // mean pooling
        let mean = hidden.mean(1)?; // [1, hidden_dim]

        // std pooling（分散の情報も加える）
        let mean_sq = hidden.sqr()?.mean(1)?; // [1, hidden_dim]
        let std = (mean_sq - mean.sqr()?)?.sqrt()?; // [1, hidden_dim]

        // mean と std を結合 → [1, hidden_dim * 2]
        let pooled = Tensor::cat(&[&mean, &std], 1)?;

        let vec: Vec<f32> = pooled.flatten_all()?.to_vec1()?;
        let norm = l2_normalize(vec);
        Ok(norm)
    }
}

fn l2_normalize(mut v: Vec<f32>) -> Vec<f32> {
    let norm = (v.iter().map(|x| x * x).sum::<f32>() + 1e-8).sqrt();
    for x in &mut v {
        *x /= norm;
    }
    v
}

impl AudioEncoder for WhisperEncoder {
    /// 各セグメントを独立してエンコードしてベクトル列を返す
    fn encode_segments(&self, segments: &[AudioSegmentView<'_>]) -> Vec<Vec<f32>> {
        segments
            .iter()
            .filter_map(|seg| match self.encode_one(seg) {
                Ok(v) => Some(v),
                Err(e) => {
                    eprintln!("Whisper encode failed at {:.1}s: {}", seg.start_secs, e);
                    None
                }
            })
            .collect()
    }

    fn feature_dim(&self) -> usize {
        self.hidden_dim * 2
    }

    fn required_sample_rate(&self) -> u32 {
        WHISPER_SAMPLE_RATE
    }
}
