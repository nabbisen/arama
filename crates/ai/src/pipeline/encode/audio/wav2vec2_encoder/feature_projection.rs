use candle_core::{Module, Tensor};
use candle_nn::{VarBuilder, layer_norm, linear};

/// 特徴量を Transformer の隠れ層次元 (768) に投影
pub struct FeatureProjection {
    layer_norm: candle_nn::LayerNorm,
    projection: candle_nn::Linear,
}

impl FeatureProjection {
    pub fn load(vb: VarBuilder, in_dim: usize, out_dim: usize) -> anyhow::Result<Self> {
        let ln = layer_norm(in_dim, 1e-5, vb.pp("layer_norm"))?;
        let proj = linear(in_dim, out_dim, vb.pp("projection"))?;
        Ok(Self {
            layer_norm: ln,
            projection: proj,
        })
    }

    pub fn forward(&self, x: &Tensor) -> anyhow::Result<Tensor> {
        // x: [B, C, T] -> [B, T, C]
        let x = x.transpose(1, 2)?;
        let x = self.layer_norm.forward(&x)?;
        let x = self.projection.forward(&x)?;
        Ok(x)
    }
}
