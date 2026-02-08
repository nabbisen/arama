use candle_core::{DType, Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::clip::{self, ClipConfig, ClipModel};
use iced::wgpu::naga::FastHashMap;
use swdir::DirNode;

use std::path::{Path, PathBuf};

use crate::app::utils::gallery::image_tensor::{calculate_cosine_similarity, load_image_as_tensor};

#[derive(Clone, Debug, Default)]
pub struct ImageSimilarity {
    files: FastHashMap<PathBuf, f32>,
}

impl ImageSimilarity {
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    pub fn get_score(&self, path: &Path) -> Option<f32> {
        self.files.get(path).copied()
    }

    // pub fn update_score(&mut self, path: PathBuf, score: f32) {
    //     self.files.insert(path, score);
    // }

    pub fn calculate(source: &Path, dir_node: &DirNode) -> anyhow::Result<Self> {
        let device = Device::new_cuda(0).unwrap_or(Device::Cpu); // GPUを使う場合は Device::new_cuda(0)

        // println!("1. モデルのロード");
        // 事前に openai/clip-vit-base-patch32 などから config.json と model.safetensors を入手してください
        let config = clip::ClipConfig::vit_base_patch32();
        let vb = unsafe {
            // todo: requires safetensors from openai/clip-vit-base-patch32
            VarBuilder::from_mmaped_safetensors(
                &[crate::app::SAFETENSORS_MODEL],
                DType::F32,
                &device,
            )?
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

        let files =
            calculate_dir_node(&dir_node, &source, &source_tensor, &device, &config, &model)?;

        Ok(Self { files })
    }
}

fn calculate_dir_node(
    dir_node: &DirNode,
    source: &Path,
    source_tensor: &Tensor,
    device: &Device,
    config: &ClipConfig,
    model: &ClipModel,
) -> anyhow::Result<FastHashMap<PathBuf, f32>, anyhow::Error> {
    let mut ret: FastHashMap<PathBuf, f32> = FastHashMap::default();

    let files_calculated: Result<FastHashMap<PathBuf, f32>, Box<dyn std::error::Error>> = dir_node
        .files
        .iter()
        .map(|target| {
            let target_image: Tensor = load_image_as_tensor(
                target.to_string_lossy().as_ref(),
                config.image_size,
                &device,
            )?;

            let file_tensor = model.get_image_features(&target_image)?;

            // println!("4. 類似度（コサイン類似度）の計算");
            let similarity = if source.eq(target.as_path()) {
                1.0
            } else {
                calculate_cosine_similarity(&source_tensor, &file_tensor)?
            };

            Ok((target.to_owned(), similarity))
        })
        .collect();

    match files_calculated {
        Ok(files_calculated) => ret.extend(files_calculated),
        Err(err) => {
            // todo: error handling
            eprintln!("{}", err);
        }
    }

    dir_node.sub_dirs.iter().for_each(|sub_dir| {
        let sub_dir_calculated =
            calculate_dir_node(sub_dir, source, source_tensor, device, config, model);
        match sub_dir_calculated {
            Ok(sub_dir_calculated) => ret.extend(sub_dir_calculated),
            Err(err) => {
                // todo: error handling
                eprintln!("{}", err);
            }
        }
    });

    Ok(ret)
}
