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
    pub expected_sha256: &'static str,
    pub config_expected_sha256: Option<&'static str>,
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
        let safetensors_ready = self.safetensors_path()?.exists();
        let config_ready = match self.config_expected_sha256 {
            Some(_) => self.config_json_path()?.exists(),
            None => true,
        };

        Ok(safetensors_ready && config_ready)
    }

    pub fn validate_dir(&self) -> Result<()> {
        validate_dir(&self.model_dir()?)
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
            .map_err(|err| {
                std::io::Error::other(format!("failed to convert pytorch to safetensors: {err}"))
            })?;

        Ok(())
    }

    fn model_dir(&self) -> Result<PathBuf> {
        Ok(models_dir()?.join(&self.name))
    }
}

fn models_dir() -> Result<PathBuf> {
    Ok(local_dir()?.join(MODEL_DIR))
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{ModelContainer, SourceUrl};

    fn unique_model_name(suffix: &str) -> String {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before UNIX epoch")
            .as_nanos();
        format!("test-{suffix}-{}-{nanos}", std::process::id())
    }

    #[test]
    fn ready_for_single_file_model_requires_safetensors_only() {
        let model = ModelContainer {
            name: unique_model_name("single"),
            source_url: SourceUrl::ModelSafetensors("https://example.invalid/model".to_owned()),
            expected_sha256: "unused",
            config_expected_sha256: None,
        };

        assert!(!model.clone().ready().expect("ready check"));

        let safetensors_path = model.safetensors_path().expect("safetensors path");
        fs::create_dir_all(safetensors_path.parent().expect("model dir"))
            .expect("create model dir");
        fs::write(&safetensors_path, b"model").expect("write safetensors");

        assert!(model.clone().ready().expect("ready check"));
        fs::remove_dir_all(safetensors_path.parent().expect("model dir"))
            .expect("cleanup model dir");
    }

    #[test]
    fn ready_for_config_model_requires_safetensors_and_config() {
        let model = ModelContainer {
            name: unique_model_name("config"),
            source_url: SourceUrl::ModelSafetensorsConfigJson((
                "https://example.invalid/model".to_owned(),
                "https://example.invalid/config".to_owned(),
            )),
            expected_sha256: "unused",
            config_expected_sha256: Some("unused"),
        };

        assert!(!model.clone().ready().expect("ready check"));

        let safetensors_path = model.safetensors_path().expect("safetensors path");
        fs::create_dir_all(safetensors_path.parent().expect("model dir"))
            .expect("create model dir");
        fs::write(&safetensors_path, b"model").expect("write safetensors");

        assert!(!model.clone().ready().expect("ready check"));

        let config_path = model.config_json_path().expect("config path");
        fs::write(&config_path, b"{}").expect("write config");

        assert!(model.clone().ready().expect("ready check"));
        fs::remove_dir_all(safetensors_path.parent().expect("model dir"))
            .expect("cleanup model dir");
    }
}
