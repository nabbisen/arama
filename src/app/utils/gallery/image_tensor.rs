use candle_core::{Device, Tensor};
use image::GenericImageView;

/// 画像ファイルをリサイズ・正規化してTensorに変換
pub fn load_image_as_tensor(path: &str, size: usize, device: &Device) -> anyhow::Result<Tensor> {
    let img = image::open(path)?;
    let img = img.resize_exact(
        size as u32,
        size as u32,
        image::imageops::FilterType::Triangle,
    );

    // CLIP標準の正規化パラメータ
    let mean = [0.48145466, 0.4578275, 0.40821073];
    let std = [0.26862954, 0.26130258, 0.27577711];

    let mut pixels = Vec::with_capacity(3 * size * size);
    for c in 0..3 {
        for y in 0..size {
            for x in 0..size {
                let p = img.get_pixel(x as u32, y as u32);
                let val = (p[c] as f32 / 255.0 - mean[c]) / std[c];
                pixels.push(val);
            }
        }
    }

    let tensor = Tensor::from_vec(pixels, (1, 3, size, size), device)?;
    Ok(tensor)
}

/// コサイン類似度の計算: (A・B) / (||A|| * ||B||)
pub fn calculate_cosine_similarity(emb1: &Tensor, emb2: &Tensor) -> anyhow::Result<f32> {
    let emb1 = emb1.flatten_all()?;
    let emb2 = emb2.flatten_all()?;

    let dot_product = (&emb1 * &emb2)?.sum_all()?.to_scalar::<f32>()?;
    let norm1 = emb1.sqr()?.sum_all()?.sqrt()?.to_scalar::<f32>()?;
    let norm2 = emb2.sqr()?.sum_all()?.sqrt()?.to_scalar::<f32>()?;

    Ok(dot_product / (norm1 * norm2))
}
