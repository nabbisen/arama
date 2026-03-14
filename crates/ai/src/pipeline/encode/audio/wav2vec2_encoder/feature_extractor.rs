use candle_core::{Module, Tensor};
use candle_nn::{Conv1dConfig, VarBuilder, conv1d, conv1d_no_bias};

use super::super::wav2vec2_config::Wav2vec2Config;

/// 7層の畳み込みによる特徴抽出器
pub struct FeatureExtractor {
    conv_layers: Vec<candle_nn::Conv1d>,
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

            // config.conv_bias が false の場合、こちらを使用する
            let conv = if !config.conv_bias {
                // バイアスなしの畳み込み層をロード
                conv1d_no_bias(
                    in_c,
                    config.conv_dim[i],
                    config.conv_kernel[i],
                    cfg,
                    vb.pp(i).pp("conv"),
                )?
            } else {
                // バイアスあり（通常の Wav2vec2 では稀ですが config に従う場合）
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
        Ok(Self { conv_layers })
    }

    pub fn forward(&self, x: &Tensor) -> anyhow::Result<Tensor> {
        let mut x = x.clone();
        for conv in &self.conv_layers {
            x = conv.forward(&x)?.gelu()?;
        }
        Ok(x)
    }
}
