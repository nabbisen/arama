use candle_core::{DType, Device, Tensor};
use candle_nn::VarBuilder;

// --- Model component definitions ---
mod feature_extractor;
mod feature_projection;

use super::wav2vec2_config::Wav2vec2Config;
use feature_extractor::FeatureExtractor;
use feature_projection::FeatureProjection;

use crate::{
    model::model_container::wav2vec2,
    pipeline::{
        encode::audio::AudioEncoder, extract::video_extractor::audio_segment::AudioSegmentView,
    },
};

// --- Main encoder implementation ---

pub struct Wav2vec2Encoder {
    feature_extractor: FeatureExtractor,
    feature_projection: FeatureProjection,
    // A full wav2vec2 stack would place 12 Transformer blocks here.
    // This skeleton keeps the high-level feature extraction flow explicit.
    device: Device,
    feature_dim: usize,
}

impl Wav2vec2Encoder {
    pub fn load(device: Device) -> anyhow::Result<Self> {
        let model = wav2vec2::model();
        let model_safetensors_path = model.safetensors_path()?;

        // Load config.json.
        let config_str = std::fs::read_to_string(model.config_json_path()?)?;
        let config: Wav2vec2Config = serde_json::from_str(&config_str)?;

        let vb = unsafe {
            VarBuilder::from_mmaped_safetensors(&[model_safetensors_path], DType::F32, &device)?
        };
        let w2v_vb = vb.pp("wav2vec2");

        let feature_extractor = FeatureExtractor::load(w2v_vb.pp("feature_extractor"), &config)?;

        let feature_projection =
            FeatureProjection::load(w2v_vb.pp("feature_projection"), 512, config.hidden_size)?;

        Ok(Self {
            feature_extractor,
            feature_projection,
            device,
            feature_dim: config.hidden_size,
        })
    }

    fn encode_one(&self, seg: &AudioSegmentView) -> anyhow::Result<Vec<f32>> {
        // Convert to [1, 1, sequence length].
        let tensor = Tensor::from_slice(seg.samples, (1, 1, seg.samples.len()), &self.device)?;

        // 1. Feature Extraction (CNN)
        let feats = self.feature_extractor.forward(&tensor)?;

        // 2. Projection (Linear) -> [1, T, 768]
        let projected = self.feature_projection.forward(&feats)?;

        // 3. A full model would pass this through the Transformer encoder;
        // this skeleton pools instead.
        // Average over the time axis (dim 1).
        let pooled = projected.mean(1)?.squeeze(0)?;

        // 4. L2 normalize.
        let vec = pooled.to_vec1::<f32>()?;
        Ok(l2_normalize(vec))
    }
}

// --- Trait implementation ---

impl AudioEncoder for Wav2vec2Encoder {
    fn encode_segments(&self, segments: &[AudioSegmentView<'_>]) -> Vec<Vec<f32>> {
        segments
            .iter()
            .map(|seg| {
                self.encode_one(seg)
                    .unwrap_or_else(|_| vec![0.0; self.feature_dim])
            })
            .collect()
    }

    fn feature_dim(&self) -> usize {
        self.feature_dim
    }

    fn required_sample_rate(&self) -> u32 {
        16000
    }
}

fn l2_normalize(mut v: Vec<f32>) -> Vec<f32> {
    let norm = (v.iter().map(|x| x * x).sum::<f32>() + 1e-8).sqrt();
    for x in &mut v {
        *x /= norm;
    }
    v
}
