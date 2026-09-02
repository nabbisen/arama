use candle_core::{Module, Tensor};
use candle_nn::{Conv1dConfig, GroupNorm, VarBuilder, conv1d_no_bias, group_norm};

use super::super::wav2vec2_config::Wav2vec2Config;

/// PyTorch's `nn.GroupNorm` default `eps` - HF's `Wav2Vec2GroupNormConvLayer`
/// constructs its `GroupNorm` without overriding it. Not the same field as
/// `config.json`'s `layer_norm_eps` (`1e-05`, coincidentally close): that
/// one belongs to the transformer's `LayerNorm`, a different layer this
/// crate does not implement.
const GROUP_NORM_EPS: f64 = 1e-5;

/// Feature extractor with 7 convolution layers.
pub struct FeatureExtractor {
    conv_layers: Vec<candle_nn::Conv1d>,
    /// Task 042 (audit A2): only present, and only ever applied to conv
    /// layer 0's output, when `feat_extract_norm` is `"group"` - the
    /// pinned model's own `config.json` value. HF's
    /// `Wav2Vec2GroupNormConvLayer` is `conv -> GroupNorm -> activation`,
    /// on layer 0 only; layers 1..N stay `conv -> activation`.
    conv0_group_norm: Option<GroupNorm>,
}

impl FeatureExtractor {
    pub fn load(vb: VarBuilder, config: &Wav2vec2Config) -> anyhow::Result<Self> {
        let mut conv_layers = Vec::new();
        let vb = vb.pp("conv_layers");

        let mut in_c = 1;
        for i in 0..config.conv_dim.len() {
            let cfg = Conv1dConfig {
                stride: config.conv_stride[i],
                ..Default::default()
            };

            // Use this path when config.conv_bias is false.
            let conv = if !config.conv_bias {
                // Load a convolution layer without bias.
                conv1d_no_bias(
                    in_c,
                    config.conv_dim[i],
                    config.conv_kernel[i],
                    cfg,
                    vb.pp(i).pp("conv"),
                )?
            } else {
                // Load a biased convolution layer when required by the config.
                candle_nn::conv1d(
                    in_c,
                    config.conv_dim[i],
                    config.conv_kernel[i],
                    cfg,
                    vb.pp(i).pp("conv"),
                )?
            };

            conv_layers.push(conv);
            in_c = config.conv_dim[i];
        }

        // Task 042 (audit A2): `wav2vec2.feature_extractor.conv_layers.0.layer_norm.{weight,bias}`
        // are real, present tensors in the shipped weights - confirmed
        // directly against the file's own header, not assumed. Confirm
        // the config's own claim about which mode this is rather than
        // silently assuming "group" - a config drift here should fail
        // loudly, not train wrongly-composed features.
        let conv0_group_norm = match config.feat_extract_norm.as_str() {
            "group" => {
                let dim = config.conv_dim[0];
                Some(group_norm(
                    dim,
                    dim,
                    GROUP_NORM_EPS,
                    vb.pp(0).pp("layer_norm"),
                )?)
            }
            other => {
                anyhow::bail!(
                    "unsupported feat_extract_norm {other:?} - arama's pinned wav2vec2-base-960h config declares \"group\", and only that mode is implemented"
                );
            }
        };

        Ok(Self {
            conv_layers,
            conv0_group_norm,
        })
    }

    pub fn forward(&self, x: &Tensor) -> anyhow::Result<Tensor> {
        let mut x = x.clone();
        for (i, conv) in self.conv_layers.iter().enumerate() {
            x = conv.forward(&x)?;
            if i == 0
                && let Some(norm) = &self.conv0_group_norm
            {
                x = norm.forward(&x)?;
            }
            x = x.gelu()?;
        }
        Ok(x)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use candle_core::{DType, Device};
    use candle_nn::VarBuilder;

    use super::{FeatureExtractor, Wav2vec2Config};

    fn config(conv_dim: Vec<usize>, feat_extract_norm: &str) -> Wav2vec2Config {
        let n = conv_dim.len();
        Wav2vec2Config {
            conv_bias: false,
            conv_dim,
            conv_kernel: vec![1; n],
            conv_stride: vec![1; n],
            hidden_size: 0,
            feat_extract_norm: feat_extract_norm.to_owned(),
        }
    }

    /// Task 042 (audit A2): conv layer 0's `GroupNorm` normalises its
    /// input to zero mean / unit variance *per channel* (num_groups ==
    /// num_channels, matching HF), before the affine `weight`/`bias` and
    /// the activation. With `weight = 1` and a deliberately huge
    /// `bias = 1000`, the post-norm, pre-activation values must cluster
    /// tightly around 1000 regardless of what the raw convolution
    /// produced - GELU is close to the identity there, so the final
    /// output stays in that band too. Without the fix (bare `conv ->
    /// gelu`, no normalisation), layer 0's output is `gelu([1,2,3,4,5])`,
    /// nowhere near 1000. This is the fix's own failing-before/passing-
    /// after test.
    #[test]
    fn conv_layer_0_group_norm_is_applied_when_config_declares_group() {
        let device = Device::Cpu;
        // Two output channels, kernel 1, so the conv is a pure per-
        // timestep scale (weight = 1 => copies the input through).
        let conv0_weight =
            candle_core::Tensor::from_slice(&[1.0f32, 1.0], (2, 1, 1), &device).unwrap();
        let norm_weight = candle_core::Tensor::from_slice(&[1.0f32, 1.0], 2, &device).unwrap();
        let norm_bias = candle_core::Tensor::from_slice(&[1000.0f32, 1000.0], 2, &device).unwrap();

        let mut tensors = HashMap::new();
        tensors.insert("conv_layers.0.conv.weight".to_owned(), conv0_weight);
        tensors.insert("conv_layers.0.layer_norm.weight".to_owned(), norm_weight);
        tensors.insert("conv_layers.0.layer_norm.bias".to_owned(), norm_bias);
        let vb = VarBuilder::from_tensors(tensors, DType::F32, &device);

        let extractor = FeatureExtractor::load(vb, &config(vec![2], "group")).unwrap();
        let input =
            candle_core::Tensor::from_slice(&[1.0f32, 2.0, 3.0, 4.0, 5.0], (1, 1, 5), &device)
                .unwrap();
        let output = extractor.forward(&input).unwrap();
        let values = output.flatten_all().unwrap().to_vec1::<f32>().unwrap();

        for v in values {
            assert!(
                (990.0..=1010.0).contains(&v),
                "expected every value clustered near the GroupNorm bias (1000), got {v} - \
                 GroupNorm was not applied to conv layer 0"
            );
        }
    }

    /// Task 042 (audit A2): the audit's own claim, made a structural
    /// assertion. `VarBuilder::from_tensors` errors on any tensor it is
    /// asked for that is not in the map - deliberately omitting
    /// `conv_layers.1.layer_norm.*` here means `load` only succeeds if
    /// layer 1 never asks for it. If a future change normalised every
    /// layer instead of only layer 0, this test would fail to load at
    /// all, not just produce a wrong number.
    #[test]
    fn only_conv_layer_0_reads_a_group_norm_layers_1_and_up_do_not() {
        let device = Device::Cpu;
        let conv0_weight =
            candle_core::Tensor::from_slice(&[1.0f32, 1.0], (2, 1, 1), &device).unwrap();
        let norm_weight = candle_core::Tensor::from_slice(&[1.0f32, 1.0], 2, &device).unwrap();
        let norm_bias = candle_core::Tensor::from_slice(&[0.0f32, 0.0], 2, &device).unwrap();
        // out=2, in=2, kernel=1, identity-like pass-through.
        let conv1_weight =
            candle_core::Tensor::from_slice(&[1.0f32, 0.0, 0.0, 1.0], (2, 2, 1), &device).unwrap();

        let mut tensors = HashMap::new();
        tensors.insert("conv_layers.0.conv.weight".to_owned(), conv0_weight);
        tensors.insert("conv_layers.0.layer_norm.weight".to_owned(), norm_weight);
        tensors.insert("conv_layers.0.layer_norm.bias".to_owned(), norm_bias);
        tensors.insert("conv_layers.1.conv.weight".to_owned(), conv1_weight);
        // No conv_layers.1.layer_norm.{weight,bias} - deliberately.
        let vb = VarBuilder::from_tensors(tensors, DType::F32, &device);

        let result = FeatureExtractor::load(vb, &config(vec![2, 2], "group"));

        assert!(
            result.is_ok(),
            "loading must not require a layer_norm for conv layer 1: {:?}",
            result.err()
        );
    }

    /// Task 042 (audit A2): a config declaring anything other than the
    /// one mode this crate implements must fail loudly at load time, not
    /// silently skip normalisation.
    #[test]
    fn an_unrecognised_feat_extract_norm_fails_to_load_rather_than_silently_skipping_it() {
        let device = Device::Cpu;
        let conv0_weight = candle_core::Tensor::from_slice(&[1.0f32], (1, 1, 1), &device).unwrap();
        let mut tensors = HashMap::new();
        tensors.insert("conv_layers.0.conv.weight".to_owned(), conv0_weight);
        let vb = VarBuilder::from_tensors(tensors, DType::F32, &device);

        let result = FeatureExtractor::load(vb, &config(vec![1], "layer"));

        assert!(result.is_err());
    }
}
