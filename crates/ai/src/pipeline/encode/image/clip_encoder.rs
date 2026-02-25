// use std::path::PathBuf;

use anyhow::Context;
use candle_core::{DType, Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::clip::{
    ClipConfig,
    ClipModel,
    // text_model::ClipTextConfig, vision_model::ClipVisionConfig,
};
use image::GenericImageView;

use crate::{
    model::model_container::clip, pipeline::extract::video_extractor::image_frame::RawVideoFrame,
};
// use naga::FastHashMap;

// CLIP の ImageNet 正規化定数
const CLIP_MEAN: [f32; 3] = [0.48145466, 0.4578275, 0.40821073];
const CLIP_STD: [f32; 3] = [0.26862954, 0.26130258, 0.27577711];

pub struct ClipEncoder {
    // pub source: PathBuf,
    // pub source_tensor: Tensor,
    pub model: ClipModel,
    pub config: ClipConfig,
    pub device: Device,
}

impl ClipEncoder {
    pub fn load(device: Device) -> anyhow::Result<Self> {
        // info!("Loading CLIP model: {}", CLIP_MODEL_ID);

        // let api = Api::new()?;
        // let repo = api.repo(Repo::new(CLIP_MODEL_ID.into(), RepoType::Model));

        // let model_file = repo.get("model.safetensors").await?;
        // let config_file = repo.get("config.json").await?;
        let model = clip::model();
        let model_safetensors_path = model.safetensors_path()?;

        // let config_json_path = model.config_json_path()?;

        // let cfg_str = std::fs::read_to_string(&config_json_path)?;
        // let cfg_val: serde_json::Value = serde_json::from_str(&cfg_str)?;
        // let config = Self::build_config(&cfg_val)?;
        let config = ClipConfig::vit_base_patch32();

        let vb = unsafe {
            VarBuilder::from_mmaped_safetensors(&[model_safetensors_path], DType::F32, &device)?
        };

        let model = ClipModel::new(vb, &config).context("Failed to load CLIP model")?;

        // todo: delete debugger
        println!("CLIP model loaded on {:?}", device);

        Ok(Self {
            model,
            config,
            device,
        })
    }

    // fn build_config(v: &serde_json::Value) -> anyhow::Result<ClipConfig> {
    //     let get_u = |path: &[&str], default: u64| -> usize {
    //         let mut cur = v;
    //         for &key in path {
    //             cur = &cur[key];
    //         }
    //         cur.as_u64().unwrap_or(default) as usize
    //     };

    //     let vision_cfg = ClipVisionConfig {
    //         embed_dim: get_u(&["projection_dim"], 512),
    //         activation: candle_transformers::models::clip::text_model::Activation::QuickGelu,
    //         intermediate_size: get_u(&["vision_config", "intermediate_size"], 2048),
    //         max_position_embeddings: 50,
    //         num_attention_heads: get_u(&["vision_config", "num_attention_heads"], 12),
    //         num_hidden_layers: get_u(&["vision_config", "num_hidden_layers"], 12),
    //         num_channels: 3,
    //         image_size: get_u(&["vision_config", "image_size"], 224),
    //         patch_size: get_u(&["vision_config", "patch_size"], 32),
    //         hidden_size: get_u(&["vision_config", "hidden_size"], 768),
    //     };

    //     let text_cfg = ClipTextConfig {
    //         vocab_size: get_u(&["text_config", "vocab_size"], 49408),
    //         embed_dim: get_u(&["projection_dim"], 512),
    //         activation: candle_transformers::models::clip::Activation::QuickGelu,
    //         intermediate_size: get_u(&["text_config", "intermediate_size"], 2048),
    //         max_position_embeddings: get_u(&["text_config", "max_position_embeddings"], 77),
    //         num_attention_heads: get_u(&["text_config", "num_attention_heads"], 8),
    //         num_hidden_layers: get_u(&["text_config", "num_hidden_layers"], 12),
    //         hidden_size: get_u(&["text_config", "hidden_size"], 512),
    //         pad_with_eos: false,
    //     };

    //     Ok(ClipConfig {
    //         text_config: text_cfg,
    //         vision_config: vision_cfg,
    //         logit_scale_init_value: 2.6592,
    //     })
    // }

    /// 複数フレームをバッチエンコードして L2 正規化済みベクトル列を返す
    pub fn encode_frames(&self, frames: &[RawVideoFrame]) -> anyhow::Result<Vec<Vec<f32>>> {
        if frames.is_empty() {
            return Ok(vec![]);
        }
        let batch = self.frames_to_tensor(frames)?;
        let embeds = self.model.get_image_features(&batch)?;
        let normed = self.l2_normalize(&embeds)?;
        self.tensor_to_vecs(normed)
    }

    /// RGB24 HWC フレーム群を CLIP 入力テンソル [B, 3, H, W] に変換する
    fn frames_to_tensor(&self, frames: &[RawVideoFrame]) -> anyhow::Result<Tensor> {
        let size = frames[0].width as usize;
        let mut data: Vec<f32> = Vec::with_capacity(frames.len() * 3 * size * size);

        for frame in frames {
            // HWC → CHW + CLIP 正規化
            for c in 0..3usize {
                let mean = CLIP_MEAN[c];
                let std = CLIP_STD[c];
                for hw in 0..(size * size) {
                    let raw = frame.data[hw * 3 + c] as f32 / 255.0;
                    data.push((raw - mean) / std);
                }
            }
        }

        Tensor::from_vec(data, (frames.len(), 3, size, size), &self.device)
            .context("CLIP input tensor construction failed")
    }

    fn l2_normalize(&self, t: &Tensor) -> anyhow::Result<Tensor> {
        let norm = (t.sqr()?.sum_keepdim(1)? + 1e-8_f64)?.sqrt()?;
        Ok(t.broadcast_div(&norm)?)
    }

    fn tensor_to_vecs(&self, t: Tensor) -> anyhow::Result<Vec<Vec<f32>>> {
        let (_batch, dim) = t.dims2()?;
        let flat: Vec<f32> = t.flatten_all()?.to_vec1()?;
        Ok(flat.chunks(dim).map(|c| c.to_vec()).collect())
    }
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

// matmul 総当り計算 対象数が少ない時限定
// pub fn find_similar_pairs(
//     map: &FastHashMap<PathBuf, Vec<f32>>,
//     threshold: f32,
// ) -> anyhow::Result<Vec<(PathBuf, PathBuf, f32)>> {
//     // 1. パスとベクトルを分離して順序を固定
//     let (paths, vectors): (Vec<&PathBuf>, Vec<Vec<f32>>) =
//         map.iter().map(|(k, v)| (k, v.clone())).unzip();

//     let n = vectors.len();
//     if n == 0 {
//         return Ok(vec![]);
//     }
//     let dim = vectors[0].len(); // CLIPなら 512 or 768

//     // 2. 2次元Tensorを作成 (N, Dim)
//     // FlattenしてTensor化
//     let flattened: Vec<f32> = vectors.into_iter().flatten().collect();
//     let tensor = Tensor::from_vec(flattened, (n, dim), &Device::Cpu)?; // 必要に応じてCudaへ

//     // 3. 行列積を計算 (N, Dim) @ (Dim, N) -> (N, N)
//     // 正規化済みなので、内積 = コサイン類似度
//     let similarity_matrix = tensor.matmul(&tensor.t()?)?;

//     // 4. 結果を解析 (ここが重くならないよう注意)
//     // 行列全体を舐めるのではなく、必要な部分だけ抽出するのが理想ですが、
//     // Rust側で処理するために一度Vecに落とす例です。
//     let scores: Vec<f32> = similarity_matrix.flatten_all()?.to_vec1()?;

//     let mut ret = Vec::new();

//     // 上三角行列だけチェック（重複と自分自身を除外）
//     for i in 0..n {
//         for j in (i + 1)..n {
//             let score = scores[i * n + j];
//             if score >= threshold {
//                 ret.push((paths[i].clone(), paths[j].clone(), score));
//             }
//         }
//     }

//     ret.sort_by(|(_, _, similarity_a), (_, _, similarity_b)| similarity_b.total_cmp(similarity_a));

//     Ok(ret)
// }
