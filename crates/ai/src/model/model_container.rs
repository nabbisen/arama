use std::path::PathBuf;

use super::{CONFIG_JSON, MODEL_DIR, PYTORCH_MODEL, SAFETENSORS_MODEL};
use arama_env::{local_dir, validate_dir};

pub mod clip;
pub mod wav2vec2;

pub enum SourceUrl {
    ModelSafetensors(String),
    ModelSafetensorsConfigJson((String, String)),
    Other(String),
}

pub struct ModelContainer {
    pub name: String,
    pub source_url: SourceUrl,
}

impl ModelContainer {
    pub fn safetensors_path(&self) -> anyhow::Result<PathBuf> {
        Ok(self.model_dir()?.join(SAFETENSORS_MODEL))
    }

    pub fn config_json_path(&self) -> anyhow::Result<PathBuf> {
        Ok(self.model_dir()?.join(CONFIG_JSON))
    }

    pub fn pytorch_path(&self) -> anyhow::Result<PathBuf> {
        Ok(self.model_dir()?.join(PYTORCH_MODEL))
    }

    pub fn ready(self) -> anyhow::Result<bool> {
        Ok(self.safetensors_path()?.exists())
    }

    pub fn validate_dir(&self) -> anyhow::Result<()> {
        Ok(validate_dir(&self.model_dir()?)?)
    }

    fn model_dir(&self) -> anyhow::Result<PathBuf> {
        Ok(models_dir()?.join(&self.name))
    }
}

fn models_dir() -> anyhow::Result<PathBuf> {
    Ok(local_dir()?.join(MODEL_DIR))
}
