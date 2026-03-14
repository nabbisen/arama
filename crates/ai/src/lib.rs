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
