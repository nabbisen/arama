use std::{
    io::Result,
    path::{Path, PathBuf},
};

use super::{CONFIG_JSON, MODEL_DIR, PYTORCH_MODEL, SAFETENSORS_MODEL};
use arama_env::local_dir;
use sha2::{Digest, Sha256};

use specification::validate_model_specification;
use transfer::sha256_hex;

pub mod clip;
mod lifecycle;
mod publication;
mod specification;
mod transfer;
pub mod wav2vec2;

// Task 036: `lifecycle` is private (its other items are implementation
// detail, reached only through `ModelContainer`'s own methods), but
// `DownloadProgress` is part of `download_with_progress`'s public
// return type and must be nameable from outside this crate.
pub use lifecycle::DownloadProgress;

const GENERATION_MANIFEST: &str = ".arama-model-generation";
const OPERATION_METADATA: &str = ".arama-model-operation";

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
    name: String,
    source_url: SourceUrl,
    expected_sha256: &'static str,
    config_expected_sha256: Option<&'static str>,
    max_model_bytes: u64,
    max_config_bytes: Option<u64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModelDownloadStatus {
    Idle,
    Downloading,
    Ready,
    Failed,
}

impl ModelContainer {
    pub fn new(
        name: impl Into<String>,
        source_url: SourceUrl,
        expected_sha256: &'static str,
        config_expected_sha256: Option<&'static str>,
        max_model_bytes: u64,
        max_config_bytes: Option<u64>,
    ) -> anyhow::Result<Self> {
        let model = Self {
            name: name.into(),
            source_url,
            expected_sha256,
            config_expected_sha256,
            max_model_bytes,
            max_config_bytes,
        };
        validate_model_specification(&model)?;
        Ok(model)
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn source_url(&self) -> &SourceUrl {
        &self.source_url
    }

    pub fn expected_sha256(&self) -> &'static str {
        self.expected_sha256
    }

    pub fn config_expected_sha256(&self) -> Option<&'static str> {
        self.config_expected_sha256
    }

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
        Ok(self.ready_in(&self.model_dir()?))
    }

    fn model_dir(&self) -> Result<PathBuf> {
        Ok(models_dir()?.join(&self.name))
    }

    fn ready_in(&self, directory: &Path) -> bool {
        directory.join(SAFETENSORS_MODEL).is_file()
            && (self.config_expected_sha256.is_none() || directory.join(CONFIG_JSON).is_file())
            && std::fs::read_to_string(directory.join(GENERATION_MANIFEST))
                .is_ok_and(|manifest| manifest == self.identity())
    }

    fn identity(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.name.as_bytes());
        match &self.source_url {
            SourceUrl::ModelSafetensors(url) => {
                hasher.update(b"safetensors\0");
                hasher.update(url.as_bytes());
            }
            SourceUrl::ModelSafetensorsConfigJson((model, config)) => {
                hasher.update(b"safetensors-config\0");
                hasher.update(model.as_bytes());
                hasher.update(b"\0");
                hasher.update(config.as_bytes());
            }
            SourceUrl::PyTorch(url) => {
                hasher.update(b"pytorch\0");
                hasher.update(url.as_bytes());
            }
        }
        hasher.update(self.expected_sha256.as_bytes());
        hasher.update(self.config_expected_sha256.unwrap_or_default().as_bytes());
        hasher.update(self.max_model_bytes.to_le_bytes());
        hasher.update(self.max_config_bytes.unwrap_or_default().to_le_bytes());
        sha256_hex(&hasher.finalize())
    }
}

fn models_dir() -> Result<PathBuf> {
    Ok(local_dir()?.join(MODEL_DIR))
}

#[cfg(test)]
use lifecycle::{
    download_entry, finish_generation, select_generation, supervise_generation, wait_for_generation,
};
#[cfg(test)]
use publication::{
    PublishFilesystem, acquire_model_lock, acquire_model_lock_with_timeout,
    next_operation_sequence, publish_generation_with, reconcile_generations,
    reconcile_generations_with,
};
#[cfg(test)]
#[path = "model_container/tests.rs"]
mod tests;
