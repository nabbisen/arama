use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Wav2vec2Config {
    pub conv_bias: bool,
    pub conv_dim: Vec<usize>,
    pub conv_kernel: Vec<usize>,
    pub conv_stride: Vec<usize>,
    pub hidden_size: usize,
    // pub num_hidden_layers: usize,
    // pub num_attention_heads: usize,
    // pub intermediate_size: usize,
    // pub layer_norm_eps: f64,
    // 必要に応じて他のフィールドも追加
}
