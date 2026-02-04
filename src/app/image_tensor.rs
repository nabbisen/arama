use std::path::{Path, PathBuf};

use candle_core::{DType, Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::clip;
use image::GenericImageView;

#[derive(Debug)]
pub struct ImageTensor {
    // source: PathBuf,
    // source_tensor: Tensor,
    pub targets: anyhow::Result<Vec<(PathBuf, f32)>>,
}

impl ImageTensor {
    pub fn new(source: &Path, targets: Vec<&Path>) -> anyhow::Result<Self> {
        let device = Device::new_cuda(0).unwrap_or(Device::Cpu); // GPUを使う場合は Device::new_cuda(0)

        // println!("1. モデルのロード");
        // 事前に openai/clip-vit-base-patch32 などから config.json と model.safetensors を入手してください
        let config = clip::ClipConfig::vit_base_patch32();
        let vb = unsafe {
            // todo: requires safetensors from openai/clip-vit-base-patch32
            VarBuilder::from_mmaped_safetensors(&["models/model.safetensors"], DType::F32, &device)?
        };
        let model = clip::ClipModel::new(vb, &config)?;

        let source = source.to_path_buf();
        // println!("2. ソース画像をロードして前処理");
        let source_image: Tensor = load_image_as_tensor(
            source.to_string_lossy().as_ref(),
            config.image_size,
            &device,
        )?;

        // println!("3. 特徴ベクトル（Embedding）の抽出");
        // [1, 3, 224, 224] -> [1, 512] (モデルにより次元は異なります)
        let source_tensor = model.get_image_features(&source_image)?;

        let targets = targets
            .iter()
            .map(|target| {
                let target = target.to_path_buf();

                let target_image: Tensor = load_image_as_tensor(
                    target.to_string_lossy().as_ref(),
                    config.image_size,
                    &device,
                )?;

                let target_tensor = model.get_image_features(&target_image)?;

                // println!("4. 類似度（コサイン類似度）の計算");
                let similarity = if source.as_path().eq(target.as_path()) {
                    1.0
                } else {
                    calculate_cosine_similarity(&source_tensor, &target_tensor)?
                };

                Ok((target, similarity))
            })
            .collect();

        Ok(Self {
            // source,
            // source_tensor,
            targets,
        })
    }
}

/// 画像ファイルをリサイズ・正規化してTensorに変換
fn load_image_as_tensor(path: &str, size: usize, device: &Device) -> anyhow::Result<Tensor> {
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
fn calculate_cosine_similarity(emb1: &Tensor, emb2: &Tensor) -> anyhow::Result<f32> {
    let emb1 = emb1.flatten_all()?;
    let emb2 = emb2.flatten_all()?;

    let dot_product = (&emb1 * &emb2)?.sum_all()?.to_scalar::<f32>()?;
    let norm1 = emb1.sqr()?.sum_all()?.sqrt()?.to_scalar::<f32>()?;
    let norm2 = emb2.sqr()?.sum_all()?.sqrt()?.to_scalar::<f32>()?;

    Ok(dot_product / (norm1 * norm2))
}
