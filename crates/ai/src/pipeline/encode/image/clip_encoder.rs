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

// CLIP ImageNet normalization constants.
const CLIP_MEAN: [f32; 3] = [0.48145466, 0.4578275, 0.40821073];
const CLIP_STD: [f32; 3] = [0.26862954, 0.261_302_6, 0.275_777_1];

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

    /// Batch-encodes frames and returns L2-normalized vectors.
    pub fn encode_frames(&self, frames: &[RawVideoFrame]) -> anyhow::Result<Vec<Vec<f32>>> {
        if frames.is_empty() {
            return Ok(vec![]);
        }
        let batch = self.frames_to_tensor(frames)?;
        let embeds = self.model.get_image_features(&batch)?;
        let normed = self.l2_normalize(&embeds)?;
        self.tensor_to_vecs(normed)
    }

    /// Converts RGB24 HWC frames to CLIP input tensor shape [B, 3, H, W].
    fn frames_to_tensor(&self, frames: &[RawVideoFrame]) -> anyhow::Result<Tensor> {
        build_frame_batch(frames, &self.device)
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

/// Builds the CLIP input batch tensor from RGB24 HWC frames, [B, 3, H, W].
///
/// Task 040 (audit A4): `frames[0]` cannot panic on an empty slice -
/// `encode_frames`, this function's only caller (via `frames_to_tensor`),
/// already returns early for an empty `frames`, before this is ever
/// reached. But a frame whose `data` does not actually match
/// `width * height * 3` (or whose `width != height`) indexes out of
/// bounds if trusted, and trusting `frames[0]`'s size for every frame
/// silently mis-slices a heterogeneous batch. Each frame is validated on
/// its own terms and a malformed one is excluded, not trusted - the
/// returned batch can therefore be shorter than `frames.len()`; the
/// caller derives the count the same way Task 042's audio fix does.
///
/// A free function, not a method, so it is testable without a loaded
/// CLIP model - `frames_to_tensor` only ever needed `self.device`.
fn build_frame_batch(frames: &[RawVideoFrame], device: &Device) -> anyhow::Result<Tensor> {
    let is_well_formed = |f: &RawVideoFrame| {
        f.width == f.height && f.data.len() == (f.width as usize) * (f.height as usize) * 3
    };
    let Some(reference) = frames.iter().find(|f| is_well_formed(f)) else {
        anyhow::bail!(
            "no well-formed frame in this batch of {} - every frame's data length disagreed \
             with its own width/height",
            frames.len()
        );
    };
    let size = reference.width as usize;
    let valid: Vec<&RawVideoFrame> = frames
        .iter()
        .filter(|f| is_well_formed(f) && f.width as usize == size)
        .collect();

    let mut data: Vec<f32> = Vec::with_capacity(valid.len() * 3 * size * size);

    for frame in &valid {
        // HWC -> CHW plus CLIP normalization.
        for c in 0..3usize {
            let mean = CLIP_MEAN[c];
            let std = CLIP_STD[c];
            for hw in 0..(size * size) {
                let raw = frame.data[hw * 3 + c] as f32 / 255.0;
                data.push((raw - mean) / std);
            }
        }
    }

    Tensor::from_vec(data, (valid.len(), 3, size, size), device)
        .context("CLIP input tensor construction failed")
}

/// Resizes and normalizes an image file, then converts it to a tensor.
pub fn load_image_as_tensor(path: &str, size: usize, device: &Device) -> anyhow::Result<Tensor> {
    let img = image::open(path)?;
    let img = img.resize_exact(
        size as u32,
        size as u32,
        image::imageops::FilterType::Triangle,
    );

    let mut pixels = Vec::with_capacity(3 * size * size);
    for c in 0..3 {
        for y in 0..size {
            for x in 0..size {
                let p = img.get_pixel(x as u32, y as u32);
                let val = (p[c] as f32 / 255.0 - CLIP_MEAN[c]) / CLIP_STD[c];
                pixels.push(val);
            }
        }
    }

    let tensor = Tensor::from_vec(pixels, (1, 3, size, size), device)?;
    Ok(tensor)
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::*;

    #[derive(Clone, Copy)]
    struct TestImageFormat {
        name: &'static str,
        suffix: &'static str,
        format: image::ImageFormat,
    }

    const ACCEPTED_IMAGE_FORMATS: &[TestImageFormat] = &[
        TestImageFormat {
            name: "PNG",
            suffix: ".png",
            format: image::ImageFormat::Png,
        },
        TestImageFormat {
            name: "JPEG",
            suffix: ".jpg",
            format: image::ImageFormat::Jpeg,
        },
        TestImageFormat {
            name: "WebP",
            suffix: ".webp",
            format: image::ImageFormat::WebP,
        },
        TestImageFormat {
            name: "GIF",
            suffix: ".gif",
            format: image::ImageFormat::Gif,
        },
        TestImageFormat {
            name: "BMP",
            suffix: ".bmp",
            format: image::ImageFormat::Bmp,
        },
    ];

    #[test]
    fn load_image_as_tensor_supports_accepted_formats() {
        for format in ACCEPTED_IMAGE_FORMATS {
            let source_dir = tempfile::TempDir::new().unwrap();
            let file = write_test_image(source_dir.path(), format);
            let tensor = load_image_as_tensor(
                file.to_str().expect("test path should be UTF-8"),
                8,
                &Device::Cpu,
            )
            .unwrap_or_else(|err| panic!("{} CLIP tensor load failed: {err}", format.name));

            assert_eq!(
                tensor.dims(),
                &[1, 3, 8, 8],
                "{} CLIP tensor shape changed",
                format.name
            );
        }
    }

    fn write_test_image(dir: &Path, format: &TestImageFormat) -> PathBuf {
        let path = dir.join(format!("fixture{}", format.suffix));
        let image = image::RgbImage::from_fn(4, 4, |x, y| {
            image::Rgb([
                (x * 31 + y * 17) as u8,
                (x * 13 + y * 47) as u8,
                (x * 19 + y * 23) as u8,
            ])
        });
        image
            .save_with_format(&path, format.format)
            .unwrap_or_else(|err| panic!("{} fixture write failed: {err}", format.name));
        path
    }

    fn well_formed_frame(size: u32) -> RawVideoFrame {
        RawVideoFrame {
            timestamp_secs: 0.0,
            width: size,
            height: size,
            data: vec![128u8; (size * size * 3) as usize],
        }
    }

    /// Task 040 (audit A4): a truncated buffer must be excluded, not
    /// indexed into - the whole point of the fix is that this no longer
    /// panics.
    #[test]
    fn build_frame_batch_excludes_a_truncated_frame_and_keeps_the_good_ones() {
        let good = well_formed_frame(4);
        let truncated = RawVideoFrame {
            timestamp_secs: 1.0,
            width: 4,
            height: 4,
            data: vec![128u8; 10], // needs 4*4*3 = 48.
        };

        let batch = build_frame_batch(&[good, truncated], &Device::Cpu)
            .expect("one well-formed frame is enough to build a batch");

        assert_eq!(
            batch.dims(),
            &[1, 3, 4, 4],
            "the truncated frame must be excluded from the batch, not indexed into"
        );
    }

    /// The audit's `width != height` case, plus the "do not trust
    /// `frames[0]`" case: the first frame here is well-formed but at a
    /// different size than the rest of a real batch would be, which
    /// `build_frame_batch` must not blindly propagate to every frame.
    #[test]
    fn build_frame_batch_excludes_a_frame_whose_width_and_height_disagree() {
        let good = well_formed_frame(4);
        let non_square = RawVideoFrame {
            timestamp_secs: 1.0,
            width: 4,
            height: 3,
            data: vec![128u8; (4 * 3 * 3) as usize], // internally consistent, just not square.
        };

        let batch = build_frame_batch(&[good, non_square], &Device::Cpu).unwrap();

        assert_eq!(batch.dims(), &[1, 3, 4, 4]);
    }

    /// When every frame is malformed, this must return a real `Err`, not
    /// index into an empty selection.
    #[test]
    fn build_frame_batch_errors_when_no_frame_is_well_formed() {
        let truncated = RawVideoFrame {
            timestamp_secs: 0.0,
            width: 4,
            height: 4,
            data: vec![128u8; 10],
        };

        assert!(build_frame_batch(&[truncated], &Device::Cpu).is_err());
    }
}

// Brute-force matmul path for small candidate sets only.
// pub fn find_similar_pairs(
//     map: &FastHashMap<PathBuf, Vec<f32>>,
//     threshold: f32,
// ) -> anyhow::Result<Vec<(PathBuf, PathBuf, f32)>> {
//     // 1. Split paths and vectors to keep deterministic ordering.
//     let (paths, vectors): (Vec<&PathBuf>, Vec<Vec<f32>>) =
//         map.iter().map(|(k, v)| (k, v.clone())).unzip();

//     let n = vectors.len();
//     if n == 0 {
//         return Ok(vec![]);
//     }
//     let dim = vectors[0].len(); // 512 or 768 for CLIP.

//     // 2. Build a 2D tensor (N, Dim).
//     // Flatten into a tensor.
//     let flattened: Vec<f32> = vectors.into_iter().flatten().collect();
//     let tensor = Tensor::from_vec(flattened, (n, dim), &Device::Cpu)?; // Move to CUDA if needed.

//     // 3. Calculate matrix product (N, Dim) @ (Dim, N) -> (N, N).
//     // Vectors are normalized, so dot product equals cosine similarity.
//     let similarity_matrix = tensor.matmul(&tensor.t()?)?;

//     // 4. Parse results carefully so this step does not dominate runtime.
//     // Ideally this would extract only the needed part instead of scanning
//     // the whole matrix; this example converts once to Vec for Rust-side processing.
//     let scores: Vec<f32> = similarity_matrix.flatten_all()?.to_vec1()?;

//     let mut ret = Vec::new();

//     // Check only the upper triangle to exclude duplicates and self-pairs.
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
