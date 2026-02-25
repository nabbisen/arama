//! OpenAI CLIP の Image Encoder を使った映像フレームの埋め込み
//!
//! Text Encoder は使用しない。
//! 出力: 512 次元 L2 正規化済みベクトル（ViT-B/32 の場合）

use anyhow::{Context, Result};
use candle_core::{DType, Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::clip::{ClipConfig, ClipModel, ClipTextConfig, ClipVisionConfig};
use hf_hub::{Repo, RepoType, api::tokio::Api};
use tracing::info;

use crate::extractor::RawVideoFrame;

const CLIP_MODEL_ID: &str = "openai/clip-vit-base-patch32";

// CLIP の ImageNet 正規化定数
const CLIP_MEAN: [f32; 3] = [0.48145466, 0.4578275, 0.40821073];
const CLIP_STD: [f32; 3] = [0.26862954, 0.26130258, 0.27577711];

pub struct ClipEncoder {
    model: ClipModel,
    device: Device,
}

impl ClipEncoder {
    /// HuggingFace Hub から CLIP モデルをロードする
    pub async fn load(device: Device) -> Result<Self> {
        info!("Loading CLIP model: {}", CLIP_MODEL_ID);

        let api = Api::new()?;
        let repo = api.repo(Repo::new(CLIP_MODEL_ID.into(), RepoType::Model));

        let model_file = repo.get("model.safetensors").await?;
        let config_file = repo.get("config.json").await?;

        let cfg_str = std::fs::read_to_string(&config_file)?;
        let cfg_val: serde_json::Value = serde_json::from_str(&cfg_str)?;
        let config = Self::build_config(&cfg_val)?;

        let vb =
            unsafe { VarBuilder::from_mmaped_safetensors(&[model_file], DType::F32, &device)? };

        let model = ClipModel::new(vb, &config).context("Failed to load CLIP model")?;

        info!("CLIP model loaded on {:?}", device);
        Ok(Self { model, device })
    }

    fn build_config(v: &serde_json::Value) -> Result<ClipConfig> {
        let get_u = |path: &[&str], default: u64| -> usize {
            let mut cur = v;
            for &key in path {
                cur = &cur[key];
            }
            cur.as_u64().unwrap_or(default) as usize
        };

        let vision_cfg = ClipVisionConfig {
            embed_dim: get_u(&["projection_dim"], 512),
            activation: candle_transformers::models::clip::Activation::QuickGelu,
            intermediate_size: get_u(&["vision_config", "intermediate_size"], 2048),
            max_position_embeddings: 50,
            num_attention_heads: get_u(&["vision_config", "num_attention_heads"], 12),
            num_hidden_layers: get_u(&["vision_config", "num_hidden_layers"], 12),
            num_channels: 3,
            image_size: get_u(&["vision_config", "image_size"], 224),
            patch_size: get_u(&["vision_config", "patch_size"], 32),
            hidden_size: get_u(&["vision_config", "hidden_size"], 768),
        };
        let text_cfg = ClipTextConfig {
            vocab_size: get_u(&["text_config", "vocab_size"], 49408),
            embed_dim: get_u(&["projection_dim"], 512),
            activation: candle_transformers::models::clip::Activation::QuickGelu,
            intermediate_size: get_u(&["text_config", "intermediate_size"], 2048),
            max_position_embeddings: get_u(&["text_config", "max_position_embeddings"], 77),
            num_attention_heads: get_u(&["text_config", "num_attention_heads"], 8),
            num_hidden_layers: get_u(&["text_config", "num_hidden_layers"], 12),
            hidden_size: get_u(&["text_config", "hidden_size"], 512),
            pad_with_eos: false,
        };

        Ok(ClipConfig {
            text_config: text_cfg,
            vision_config: vision_cfg,
            logit_scale_init_value: 2.6592,
        })
    }

    // ── 映像フレームのエンコード ──────────────────────────────────────

    /// 複数フレームをバッチエンコードして L2 正規化済みベクトル列を返す
    pub fn encode_frames(&self, frames: &[RawVideoFrame]) -> Result<Vec<Vec<f32>>> {
        if frames.is_empty() {
            return Ok(vec![]);
        }
        let batch = self.frames_to_tensor(frames)?;
        let embeds = self.model.get_image_features(&batch)?;
        let normed = self.l2_normalize(&embeds)?;
        self.tensor_to_vecs(normed)
    }

    /// RGB24 HWC フレーム群を CLIP 入力テンソル [B, 3, H, W] に変換する
    fn frames_to_tensor(&self, frames: &[RawVideoFrame]) -> Result<Tensor> {
        let size = frames[0].width as usize;
        let mut data: Vec<f32> = Vec::with_capacity(frames.len() * 3 * size * size);

        for frame in frames {
            // HWC → CHW + CLIP 正規化
            for c in 0..3usize {
                let mean = CLIP_MEAN[c];
                let std = CLIP_STD[c];
                for hw in 0..(size * size) {
                    let raw = frame.data[hw * 3 + c] as f32 / 255.0;
                    data.push((raw - mean) / std);
                }
            }
        }

        Tensor::from_vec(data, (frames.len(), 3, size, size), &self.device)
            .context("CLIP input tensor construction failed")
    }

    fn l2_normalize(&self, t: &Tensor) -> Result<Tensor> {
        let norm = (t.sqr()?.sum_keepdim(1)? + 1e-8_f64)?.sqrt()?;
        Ok(t.broadcast_div(&norm)?)
    }

    fn tensor_to_vecs(&self, t: Tensor) -> Result<Vec<Vec<f32>>> {
        let (_batch, dim) = t.dims2()?;
        let flat: Vec<f32> = t.flatten_all()?.to_vec1()?;
        Ok(flat.chunks(dim).map(|c| c.to_vec()).collect())
    }
}
