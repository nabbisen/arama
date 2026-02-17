use std::path::PathBuf;

use candle_core::{Device, Tensor};
use candle_transformers::models::clip::{ClipConfig, ClipModel};
use iced::wgpu::naga::FastHashMap;
use image::GenericImageView;
pub struct ClipCalculator {
    // pub source: PathBuf,
    // pub source_tensor: Tensor,
    pub model: ClipModel,
    pub config: ClipConfig,
    pub device: Device,
}

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

// /// コサイン類似度の計算: (A・B) / (||A|| * ||B||)
// pub fn calculate_cosine_similarity(emb1: &Tensor, emb2: &Tensor) -> anyhow::Result<f32> {
//     let emb1 = emb1.flatten_all()?;
//     let emb2 = emb2.flatten_all()?;

//     let dot_product = (&emb1 * &emb2)?.sum_all()?.to_scalar::<f32>()?;
//     let norm1 = emb1.sqr()?.sum_all()?.sqrt()?.to_scalar::<f32>()?;
//     let norm2 = emb2.sqr()?.sum_all()?.sqrt()?.to_scalar::<f32>()?;

//     Ok(dot_product / (norm1 * norm2))
// }

pub fn find_similar_pairs(
    map: &FastHashMap<PathBuf, Vec<f32>>,
    threshold: f32,
) -> anyhow::Result<Vec<(PathBuf, PathBuf, f32)>> {
    // 1. パスとベクトルを分離して順序を固定
    let paths: Vec<&PathBuf> = map.keys().collect();
    let vectors: Vec<Vec<f32>> = map.values().cloned().collect();
    let n = vectors.len();
    let dim = vectors[0].len(); // CLIPなら 512 or 768

    // 2. 2次元Tensorを作成 (N, Dim)
    // FlattenしてTensor化
    let flattened: Vec<f32> = vectors.into_iter().flatten().collect();
    let tensor = Tensor::from_vec(flattened, (n, dim), &Device::Cpu)?; // 必要に応じてCudaへ

    // 3. 行列積を計算 (N, Dim) @ (Dim, N) -> (N, N)
    // 正規化済みなので、内積 = コサイン類似度
    let similarity_matrix = tensor.matmul(&tensor.t()?)?;

    // 4. 結果を解析 (ここが重くならないよう注意)
    // 行列全体を舐めるのではなく、必要な部分だけ抽出するのが理想ですが、
    // Rust側で処理するために一度Vecに落とす例です。
    let scores: Vec<f32> = similarity_matrix.flatten_all()?.to_vec1()?;

    let mut ret = Vec::new();

    // 上三角行列だけチェック（重複と自分自身を除外）
    for i in 0..n {
        for j in (i + 1)..n {
            let score = scores[i * n + j];
            if score >= threshold {
                ret.push((paths[i].clone(), paths[j].clone(), score));
            }
        }
    }

    ret.sort_by(|(_, _, similarity_a), (_, _, similarity_b)| similarity_b.total_cmp(similarity_a));

    Ok(ret)
}
