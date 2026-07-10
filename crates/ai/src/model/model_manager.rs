use std::{fmt::Write as _, fs, path::PathBuf};

use candle_core::Device;
use sha2::{Digest, Sha256};

use super::{model_container::ModelContainer, model_container::SourceUrl};

pub struct ModelManager {
    model_container: ModelContainer,
}

impl ModelManager {
    pub fn device() -> Device {
        Device::new_cuda(0).unwrap_or(Device::new_metal(0).unwrap_or(Device::Cpu))
    }

    pub fn new(model_container: ModelContainer) -> anyhow::Result<Self> {
        let _ = &model_container.validate_dir()?;

        Ok(Self { model_container })
    }

    #[deprecated = "use ModelContainer.ensure_safetensors()"]
    pub async fn ensure(&self) -> anyhow::Result<()> {
        let (source_url, is_model_safetensors) = match &self.model_container.source_url {
            SourceUrl::ModelSafetensors(model_safetensors_url) => (model_safetensors_url, true),
            SourceUrl::ModelSafetensorsConfigJson((model_safetensors_url, config_json_url)) => {
                let response = reqwest::get(config_json_url).await?;
                let bytes = response.bytes().await?;
                verify_sha256(
                    &bytes,
                    self.model_container.config_expected_sha256,
                    "config.json",
                )?;

                let config_json_path = self.model_container.config_json_path()?;
                fs::write(&config_json_path, &bytes)?;

                (model_safetensors_url, true)
            }
            SourceUrl::PyTorch(source_url) => (source_url, false),
        };

        let response = reqwest::get(source_url).await?;
        let bytes = response.bytes().await?;
        verify_sha256(&bytes, Some(self.model_container.expected_sha256), "model")?;

        if is_model_safetensors {
            let model_safetensors_path = self.model_container.safetensors_path()?;
            fs::write(&model_safetensors_path, &bytes)?;
            return Ok(());
        }

        let pytorch_path = self.model_container.pytorch_path()?.clone();
        fs::write(&pytorch_path, &bytes)?;

        pt2safetensors::Pt2Safetensors::default()
            .removes_pt_at_conversion_success()
            .convert(pytorch_path, self.model_container.safetensors_path()?)
            .map_err(|err| anyhow::anyhow!("failed to convert pytorch to safetensors: {err}"))?;

        Ok(())
    }

    pub fn safetensors_path(&self) -> anyhow::Result<PathBuf> {
        Ok(self.model_container.safetensors_path()?)
    }
}

fn verify_sha256(
    bytes: &[u8],
    expected_sha256: Option<&str>,
    artifact_name: &str,
) -> anyhow::Result<()> {
    let Some(expected_sha256) = expected_sha256 else {
        return Ok(());
    };

    let actual = sha256_hex(bytes);
    if actual != expected_sha256 {
        anyhow::bail!(
            "{artifact_name} checksum mismatch: expected SHA-256 {expected_sha256}, got {actual}"
        );
    }

    Ok(())
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut hex = String::with_capacity(digest.len() * 2);
    for byte in digest {
        write!(&mut hex, "{byte:02x}").expect("writing to string cannot fail");
    }
    hex
}
