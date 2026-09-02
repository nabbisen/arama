use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Wav2vec2Config {
    pub conv_bias: bool,
    pub conv_dim: Vec<usize>,
    pub conv_kernel: Vec<usize>,
    pub conv_stride: Vec<usize>,
    pub hidden_size: usize,
    /// Task 042 (audit A2): the pinned model's `config.json` declares
    /// `"group"` - conv layer 0 is normalised with `GroupNorm` and
    /// layers 1..N are not. `FeatureExtractor::load` confirms this value
    /// rather than assuming it.
    pub feat_extract_norm: String,
    // pub num_hidden_layers: usize,
    // pub num_attention_heads: usize,
    // pub intermediate_size: usize,
    // pub layer_norm_eps: f64,
    // Add other fields as needed.
}

impl Wav2vec2Config {
    /// The minimum sample count that produces at least one output frame
    /// from the convolution stack, folded back from the last layer:
    /// `n = 1; for (k, s) in kernel.zip(stride).rev() { n = (n - 1) * s + k }`
    /// (review 137 §4). Below this, candle 0.11's `conv1d` panics rather
    /// than returning `Err` - `Wav2vec2Encoder::encode_one` rejects a
    /// segment shorter than this itself, before it ever reaches candle,
    /// so the failure reports its real cause instead of guessing at one.
    pub fn min_conv_input_len(&self) -> usize {
        self.conv_kernel
            .iter()
            .zip(&self.conv_stride)
            .rev()
            .fold(1usize, |n, (&k, &s)| (n - 1) * s + k)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The pinned wav2vec2-base-960h model's real `conv_kernel`/
    /// `conv_stride` (confirmed against its own `config.json` in review
    /// 137 §4) fold back to exactly 400 samples - 25 ms at 16 kHz.
    #[test]
    fn min_conv_input_len_matches_the_real_pinned_model_architecture() {
        let config = Wav2vec2Config {
            conv_bias: false,
            conv_dim: vec![512; 7],
            conv_kernel: vec![10, 3, 3, 3, 3, 2, 2],
            conv_stride: vec![5, 2, 2, 2, 2, 2, 2],
            hidden_size: 768,
            feat_extract_norm: "group".to_owned(),
        };
        assert_eq!(config.min_conv_input_len(), 400);
    }

    #[test]
    fn min_conv_input_len_for_a_single_layer_is_just_its_kernel() {
        let config = Wav2vec2Config {
            conv_bias: false,
            conv_dim: vec![2],
            conv_kernel: vec![3],
            conv_stride: vec![1],
            hidden_size: 2,
            feat_extract_norm: "group".to_owned(),
        };
        assert_eq!(config.min_conv_input_len(), 3);
    }
}
