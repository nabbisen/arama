use std::path::Path;

use candle_core::{DType, IndexOp, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::clip::{ClipConfig, ClipModel};

use crate::{
    model::model_container::clip, model::model_manager::ModelManager,
    store::file::file_embedding::FileEmbedding,
};

pub mod clip_encoder;
pub mod embeddings;

use clip_encoder::{ClipEncoder, load_image_as_tensor};

// pub fn calculator(source: &Path) -> anyhow::Result<Calculator> {
pub fn clip_calculator() -> anyhow::Result<ClipEncoder> {
    let device = ModelManager::device();

    // println!("1. Load the model");
    // Obtain config.json and model.safetensors from openai/clip-vit-base-patch32 or equivalent first.
    let config = ClipConfig::vit_base_patch32();
    let clip_model_manager = ModelManager::new(clip::model())?;
    let vb = unsafe {
        // todo: requires safetensors from openai/clip-vit-base-patch32
        VarBuilder::from_mmaped_safetensors(
            &[clip_model_manager.safetensors_path()?],
            DType::F32,
            &device,
        )?
    };
    let model = ClipModel::new(vb, &config)?;

    // let source = source.to_path_buf();
    // // println!("2. Load and preprocess the source image");
    // let source_image: Tensor = load_image_as_tensor(
    //     source.to_string_lossy().as_ref(),
    //     config.image_size,
    //     &device,
    // )?;

    // // println!("3. Extract the feature vector (embedding)");
    // // [1, 3, 224, 224] -> [1, 512] (dimension depends on the model)
    // let source_tensor = model.get_image_features(&source_image)?;

    Ok(ClipEncoder {
        // source: source.to_path_buf(),
        // source_tensor,
        model,
        config,
        device,
    })
}

pub fn clip(target: &Path, clip_calculator: &ClipEncoder) -> anyhow::Result<FileEmbedding> {
    let target_image: Tensor = load_image_as_tensor(
        target.to_string_lossy().as_ref(),
        clip_calculator.config.image_size,
        &clip_calculator.device,
    )?;

    let file_tensor = &clip_calculator.model.get_image_features(&target_image)?;

    // --- 1. Remove the batch dimension -----------------------
    // CLIP output is usually [1, D] or [B, D].
    let t = match file_tensor.dims() {
        // [D]
        [_d] => file_tensor.clone(),

        // [1, D] or [B, D] -> take the first item.
        [_b, _d] => file_tensor.i(0)?,

        // Handle rare model outputs such as [1, 1, D].
        _ => file_tensor.flatten_all()?,
    };

    // --- 2. Flatten to Vec<f32>
    let mut v = t.flatten_all()?.to_vec1::<f32>()?;

    // --- 3. L2 normalize (required) --------------------------
    let norm = v.iter().map(|x| x * x).sum::<f32>().sqrt();

    // Guard against failed inference producing a zero vector.
    if norm > 1e-12 {
        for x in &mut v {
            *x /= norm;
        }
    }

    Ok(FileEmbedding {
        path: target.to_path_buf(),
        embedding: v,
    })

    // // println!("4. Calculate similarity (cosine similarity)");
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
