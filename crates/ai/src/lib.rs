//! # arama-ai
//!
//! Offline AI inference pipeline for arama.
//!
//! ## Modules
//!
//! - [`config`] — [`VideoSimilarityConfig`](config::video_similarity_config::VideoSimilarityConfig):
//!   sampling timestamps and image/audio score weights for video analysis.
//! - [`model`] — [`ModelContainer`](model::model_container::ModelContainer)
//!   definitions for CLIP (`clip-vit-base-patch32`) and wav2vec2
//!   (`wav2vec2-base-960h`); download metadata for the HuggingFace sources.
//! - [`pipeline`] — CLIP image encoder, wav2vec2 audio encoder, and the
//!   top-level [`image_embedding`](pipeline::encode::image::embeddings::image_embedding)
//!   async function that indexes a list of paths.
//! - [`pipeline_manager`] — [`VideoSimilarityPipeline`](pipeline_manager::video_similarity_pipeline::VideoSimilarityPipeline):
//!   orchestrates frame sampling, audio segmentation, and score weighting
//!   for video files.

// todo: parameters for inference quality - where to define actually ?
const CLIP_IMAGE_SIZE: usize = 224;
const CROSS_MAX_SIMILARITY_THRESHOLD: f32 = 0.25;
const VIDEO_IMAGE_WEIGHT: f32 = 0.60;
macro_rules! video_audio_weight {
    () => {
        1.0 - VIDEO_IMAGE_WEIGHT
    };
}

pub mod config;
pub mod model;
pub mod pipeline;
pub mod pipeline_manager;
pub mod store;
