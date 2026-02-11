use candle_core::{DType, Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::clip::{self, ClipConfig, ClipModel};
use iced::wgpu::naga::FastHashMap;

use std::path::{Path, PathBuf};

use crate::app::utils::gallery::image_tensor::{calculate_cosine_similarity, load_image_as_tensor};

#[derive(Clone, Debug, Default)]
pub struct ImageSimilarityMap {
    files: FastHashMap<PathBuf, f32>,
}

#[derive(Clone, Debug)]
pub struct ImageSimilarity {
    pub path: PathBuf,
    pub score: f32,
}

pub struct Calculator {
    source: PathBuf,
    source_tensor: Tensor,
    model: ClipModel,
    config: ClipConfig,
    device: Device,
}

impl ImageSimilarityMap {
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    pub fn get_score(&self, path: &Path) -> Option<f32> {
        self.files.get(path).copied()
    }

    pub fn set_score(&mut self, path: &Path, score: f32) {
        self.files.insert(path.to_path_buf(), score);
    }
}

pub fn calculator_prepare(source: &Path) -> anyhow::Result<Calculator> {
    let device = Device::new_cuda(0).unwrap_or(Device::Cpu); // GPUを使う場合は Device::new_cuda(0)

    // println!("1. モデルのロード");
    // 事前に openai/clip-vit-base-patch32 などから config.json と model.safetensors を入手してください
    let config = clip::ClipConfig::vit_base_patch32();
    let vb = unsafe {
        // todo: requires safetensors from openai/clip-vit-base-patch32
        VarBuilder::from_mmaped_safetensors(&[crate::app::SAFETENSORS_MODEL], DType::F32, &device)?
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

    Ok(Calculator {
        source: source.to_path_buf(),
        source_tensor,
        model,
        config,
        device,
    })
}

pub fn calculate(
    target: &Path,
    calculator: &Calculator,
) -> anyhow::Result<ImageSimilarity, anyhow::Error> {
    let target_image: Tensor = load_image_as_tensor(
        target.to_string_lossy().as_ref(),
        calculator.config.image_size,
        &calculator.device,
    )?;

    let file_tensor = &calculator.model.get_image_features(&target_image)?;

    // println!("4. 類似度（コサイン類似度）の計算");
    let score = if calculator.source.as_path().eq(target) {
        1.0
    } else {
        calculate_cosine_similarity(&calculator.source_tensor, file_tensor)?
    };

    Ok(ImageSimilarity {
        path: target.to_owned(),
        score,
    })
}
