use std::{io::Result, path::PathBuf};

use super::{CONFIG_JSON, MODEL_DIR, PYTORCH_MODEL, SAFETENSORS_MODEL};
use arama_env::{local_dir, validate_dir};

pub mod clip;
pub mod wav2vec2;

#[derive(Clone, Debug)]
pub enum SourceUrl {
    ModelSafetensors(String),
    ModelSafetensorsConfigJson((String, String)),
    PyTorch(String),
}

impl SourceUrl {
    pub fn download_url(&self) -> String {
        let ret = match self {
            SourceUrl::ModelSafetensors(s) => s,
            SourceUrl::ModelSafetensorsConfigJson((s, _)) => s,
            SourceUrl::PyTorch(s) => s,
        };
        ret.to_owned()
    }
}

#[derive(Clone, Debug)]
pub struct ModelContainer {
    pub name: String,
    pub source_url: SourceUrl,
}

impl ModelContainer {
    pub fn safetensors_path(&self) -> Result<PathBuf> {
        Ok(self.model_dir()?.join(SAFETENSORS_MODEL))
    }

    pub fn config_json_path(&self) -> Result<PathBuf> {
        Ok(self.model_dir()?.join(CONFIG_JSON))
    }

    pub fn pytorch_path(&self) -> Result<PathBuf> {
        Ok(self.model_dir()?.join(PYTORCH_MODEL))
    }

    pub fn ready(self) -> Result<bool> {
        Ok(self.safetensors_path()?.exists())
    }

    pub fn validate_dir(&self) -> Result<()> {
        Ok(validate_dir(&self.model_dir()?)?)
    }

    pub fn ensure_safetensors(&self) -> Result<()> {
        let is_model_safetensors = match &self.source_url {
            SourceUrl::ModelSafetensors(_) | SourceUrl::ModelSafetensorsConfigJson(_) => true,
            SourceUrl::PyTorch(_) => false,
        };

        if is_model_safetensors {
            return Ok(());
        }

        let pytorch_path = self.pytorch_path()?.clone();

        pt2safetensors::Pt2Safetensors::default()
            .removes_pt_at_conversion_success()
            .convert(pytorch_path, self.safetensors_path()?)
            .expect("failed to convert pytorch to safetensors");

        Ok(())
    }

    fn model_dir(&self) -> Result<PathBuf> {
        Ok(models_dir()?.join(&self.name))
    }
}

fn models_dir() -> Result<PathBuf> {
    Ok(local_dir()?.join(MODEL_DIR))
}
