use std::path::Path;

use candle_core::{DType, Device, IndexOp, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::clip::{ClipConfig, ClipModel};

use crate::engine::store::file::file_embedding::FileEmbedding;

pub mod clip_calculator;

use clip_calculator::{ClipCalculator, load_image_as_tensor};

// pub fn calculator(source: &Path) -> anyhow::Result<Calculator> {
pub fn clip_calculator() -> anyhow::Result<ClipCalculator> {
    let device = Device::new_cuda(0).unwrap_or(Device::Cpu); // GPUを使う場合は Device::new_cuda(0)

    // println!("1. モデルのロード");
    // 事前に openai/clip-vit-base-patch32 などから config.json と model.safetensors を入手してください
    let config = ClipConfig::vit_base_patch32();
    let vb = unsafe {
        // todo: requires safetensors from openai/clip-vit-base-patch32
        VarBuilder::from_mmaped_safetensors(
            &[crate::engine::SAFETENSORS_MODEL],
            DType::F32,
            &device,
        )?
    };
    let model = ClipModel::new(vb, &config)?;

    // let source = source.to_path_buf();
    // // println!("2. ソース画像をロードして前処理");
    // let source_image: Tensor = load_image_as_tensor(
    //     source.to_string_lossy().as_ref(),
    //     config.image_size,
    //     &device,
    // )?;

    // // println!("3. 特徴ベクトル（Embedding）の抽出");
    // // [1, 3, 224, 224] -> [1, 512] (モデルにより次元は異なります)
    // let source_tensor = model.get_image_features(&source_image)?;

    Ok(ClipCalculator {
        // source: source.to_path_buf(),
        // source_tensor,
        model,
        config,
        device,
    })
}

pub fn clip(target: &Path, clip_calculator: &ClipCalculator) -> anyhow::Result<FileEmbedding> {
    let target_image: Tensor = load_image_as_tensor(
        target.to_string_lossy().as_ref(),
        clip_calculator.config.image_size,
        &clip_calculator.device,
    )?;

    let file_tensor = &clip_calculator.model.get_image_features(&target_image)?;

    // --- 1. バッチ次元の除去 --------------------------------
    // CLIPの出力は多くの場合 [1, D] または [B, D]
    let t = match file_tensor.dims() {
        // [D]
        [_d] => file_tensor.clone(),

        // [1, D] or [B, D] → 先頭1件を取り出す
        [_b, _d] => file_tensor.i(0)?,

        // まれに [1,1,D] などが来るモデル対策
        _ => file_tensor.flatten_all()?,
    };

    // --- 2. 1次元化して Vec<f32> へ
    let mut v = t.flatten_all()?.to_vec1::<f32>()?;

    // --- 3. L2正規化（必須） ----------------------------------
    let norm = v.iter().map(|x| x * x).sum::<f32>().sqrt();

    // 失敗推論対策（ゼロベクトル防止）
    if norm > 1e-12 {
        for x in &mut v {
            *x /= norm;
        }
    }

    Ok(FileEmbedding {
        path: target.to_path_buf(),
        embedding: v,
    })

    // // println!("4. 類似度（コサイン類似度）の計算");
    // let score = if calculator.source.as_path().eq(target) {
    //     1.0
    // } else {
    //     calculate_cosine_similarity(&calculator.source_tensor, file_tensor)?
    // };

    // Ok(ImageSimilarity {
    //     path: target.to_owned(),
    //     score,
    // })
}
