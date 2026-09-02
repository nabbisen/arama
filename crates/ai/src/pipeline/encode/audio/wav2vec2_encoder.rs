use candle_core::{DType, Device, Tensor};
use candle_nn::VarBuilder;

// --- Model component definitions ---
mod feature_extractor;
mod feature_projection;
#[cfg(test)]
mod tensor_coverage_gate;

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
    /// Task 042 (audit A10, review 137 §4): the shortest segment the
    /// conv stack can produce one output frame from, derived from
    /// `config.json`'s own `conv_kernel`/`conv_stride`
    /// (`Wav2vec2Config::min_conv_input_len`). `encode_one` rejects
    /// anything shorter *before* it reaches candle, whose `conv1d`
    /// panics rather than errors below this length.
    min_input_len: usize,
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

        let min_input_len = config.min_conv_input_len();

        Ok(Self {
            feature_extractor,
            feature_projection,
            device,
            feature_dim: config.hidden_size,
            min_input_len,
        })
    }

    fn encode_one(&self, seg: &AudioSegmentView) -> anyhow::Result<Vec<f32>> {
        // Task 042 (audit A10, review 137 §4): reject a segment too
        // short for the conv stack to produce even one output frame
        // *before* it reaches candle - candle 0.11's conv1d computes its
        // output length as an unsigned subtraction and panics rather
        // than returning `Err` below this length (confirmed directly, at
        // input lengths 0, 1 and 2 against a kernel of 3, all three
        // panicked). A short segment near the end of a clip is a real,
        // reachable case, not a contrived one, so this must report its
        // real cause and length - not a guess reverse-engineered from a
        // caught panic, which would misattribute any *other* panic
        // (a candle regression, an allocation failure) as "too short"
        // just as confidently and just as wrongly.
        if seg.samples.len() < self.min_input_len {
            anyhow::bail!(
                "segment too short to encode: {} samples, need at least {}",
                seg.samples.len(),
                self.min_input_len
            );
        }

        // Backstop only, for anything else that might panic inside
        // candle - deliberately does not claim a cause, since one is
        // not known.
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| self.encode_one_inner(seg)))
        {
            Ok(result) => result,
            Err(_) => anyhow::bail!(
                "unexpected panic while encoding a segment ({} samples)",
                seg.samples.len()
            ),
        }
    }

    fn encode_one_inner(&self, seg: &AudioSegmentView) -> anyhow::Result<Vec<f32>> {
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
            .filter_map(|seg| self.encode_one(seg).ok())
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    /// A tiny, entirely synthetic encoder - one conv layer with kernel
    /// size 3, so a segment shorter than 3 samples cannot be convolved
    /// and `encode_one` genuinely fails, without needing the real 361 MB
    /// model. Mirrors `feature_extractor::tests`' own synthetic-tensor
    /// approach.
    fn tiny_encoder() -> Wav2vec2Encoder {
        let device = Device::Cpu;
        let config = Wav2vec2Config {
            conv_bias: false,
            conv_dim: vec![2],
            conv_kernel: vec![3],
            conv_stride: vec![1],
            hidden_size: 2,
            feat_extract_norm: "group".to_owned(),
        };

        // The two output channels use different kernels deliberately: the
        // feature projection's LayerNorm normalizes *across* channels at
        // each timestep, so identical channels (as an earlier version of
        // this fixture had) collapse to zero there for the same reason a
        // constant-over-time signal collapses under GroupNorm - degenerate
        // for this fixture's own assertions, not a bug in the code.
        let mut extractor_tensors = HashMap::new();
        extractor_tensors.insert(
            "conv_layers.0.conv.weight".to_owned(),
            Tensor::from_slice(&[0.1f32, 0.1, 0.1, 0.3, 0.05, -0.05], (2, 1, 3), &device).unwrap(),
        );
        extractor_tensors.insert(
            "conv_layers.0.layer_norm.weight".to_owned(),
            Tensor::from_slice(&[1.0f32, 1.0], 2, &device).unwrap(),
        );
        extractor_tensors.insert(
            "conv_layers.0.layer_norm.bias".to_owned(),
            Tensor::from_slice(&[0.0f32, 0.0], 2, &device).unwrap(),
        );
        let extractor_vb = VarBuilder::from_tensors(extractor_tensors, DType::F32, &device);
        let feature_extractor = FeatureExtractor::load(extractor_vb, &config).unwrap();

        let mut projection_tensors = HashMap::new();
        projection_tensors.insert(
            "layer_norm.weight".to_owned(),
            Tensor::from_slice(&[1.0f32, 1.0], 2, &device).unwrap(),
        );
        projection_tensors.insert(
            "layer_norm.bias".to_owned(),
            Tensor::from_slice(&[0.0f32, 0.0], 2, &device).unwrap(),
        );
        projection_tensors.insert(
            "projection.weight".to_owned(),
            Tensor::from_slice(&[1.0f32, 0.0, 0.0, 1.0], (2, 2), &device).unwrap(),
        );
        projection_tensors.insert(
            "projection.bias".to_owned(),
            Tensor::from_slice(&[0.0f32, 0.0], 2, &device).unwrap(),
        );
        let projection_vb = VarBuilder::from_tensors(projection_tensors, DType::F32, &device);
        let feature_projection = FeatureProjection::load(projection_vb, 2, 2).unwrap();

        Wav2vec2Encoder {
            feature_extractor,
            feature_projection,
            device,
            feature_dim: 2,
            min_input_len: config.min_conv_input_len(),
        }
    }

    fn segment(samples: &[f32]) -> AudioSegmentView<'_> {
        AudioSegmentView {
            start_secs: 0.0,
            sample_rate: 16000,
            samples,
        }
    }

    /// Task 042 (audit A10): a segment too short for the smallest conv
    /// kernel (3 samples) makes `encode_one` genuinely fail - proves the
    /// *encoder's* own failure path is real, not simulated, before
    /// asserting what `encode_segments` does with it below.
    #[test]
    fn a_too_short_segment_makes_encode_one_fail() {
        let encoder = tiny_encoder();
        let too_short = [1.0f32];
        assert!(encoder.encode_one(&segment(&too_short)).is_err());
    }

    /// Review 137 §4: the precondition `encode_one` checks before ever
    /// reaching candle, at its exact boundary - one sample below
    /// `min_input_len` must be rejected, exactly `min_input_len` must be
    /// accepted. Cheaper and more precise than relying on a caught panic:
    /// this asserts the real threshold, not "somewhere short enough to
    /// panic."
    #[test]
    fn min_input_len_boundary_rejects_one_below_and_accepts_exactly_at() {
        let encoder = tiny_encoder();
        assert_eq!(encoder.min_input_len, 3);

        let one_below = [0.1f32, 0.2];
        assert!(
            encoder.encode_one(&segment(&one_below)).is_err(),
            "2 samples is one below the minimum (3) and must be rejected"
        );

        let at_minimum = [0.1f32, 0.15, 0.2];
        assert!(
            encoder.encode_one(&segment(&at_minimum)).is_ok(),
            "3 samples is exactly the minimum and must be accepted"
        );
    }

    /// The fix itself: a failed segment must be *excluded*, not turned
    /// into a zero vector. Before the fix
    /// (`unwrap_or_else(|_| vec![0.0; feature_dim])`), this returned 2
    /// vectors, the second one `[0.0, 0.0]`; after it, exactly 1.
    #[test]
    fn encode_segments_excludes_a_segment_that_failed_to_encode_rather_than_zero_filling_it() {
        let encoder = tiny_encoder();
        // A varying signal, not a constant one: a constant input has zero
        // variance across time, which this single-layer synthetic
        // encoder's GroupNorm correctly normalizes to exactly zero -
        // degenerate for this assertion, not a bug.
        let good: Vec<f32> = (0..20).map(|i| 0.1 + 0.05 * (i as f32 % 3.0)).collect();
        let too_short = [1.0f32];
        let segments = [segment(&good), segment(&too_short)];

        let result = encoder.encode_segments(&segments);

        assert_eq!(
            result.len(),
            1,
            "expected exactly the one segment that could encode, got {result:?}"
        );
        assert!(
            result[0].iter().any(|v| *v != 0.0),
            "the surviving vector must be the real encoded one, not a zero-filled placeholder"
        );
    }

    /// Task 042 (audit A10): when *every* segment fails, the caller must
    /// see an honest absence (`mean_embeddings` on an empty slice already
    /// returns `vec![]`, which `has_signal` correctly rejects) - not a
    /// zero vector that could be mistaken for a real, signal-less
    /// encoding.
    #[test]
    fn encode_segments_returns_empty_when_every_segment_fails() {
        let encoder = tiny_encoder();
        let too_short = [1.0f32];
        let segments = [segment(&too_short), segment(&too_short)];

        let result = encoder.encode_segments(&segments);

        assert!(result.is_empty());
    }
}
