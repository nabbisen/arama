mod core;

const SAFETENSORS_MODEL: &str = "model.safetensors";
const PYTORCH_MODEL: &str = "pytorch_model.bin";
const URL: &str = "https://huggingface.co/openai/clip-vit-base-patch32/resolve/main/pytorch_model.bin?download=true";

pub use core::{model, pipeline, store};
