use std::{fs, path::PathBuf};

use candle_core::Device;

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

    pub async fn ensure(&self) -> anyhow::Result<()> {
        let (source_url, is_model_safetensors) = match &self.model_container.source_url {
            SourceUrl::ModelSafetensors(model_safetensors_url) => (model_safetensors_url, true),
            SourceUrl::ModelSafetensorsConfigJson((model_safetensors_url, config_json_url)) => {
                let response = reqwest::get(config_json_url).await?;
                let bytes = response.bytes().await?;

                let config_json_path = self.model_container.config_json_path()?;
                let _ = fs::write(&config_json_path, &bytes)?;

                (model_safetensors_url, true)
            }
            SourceUrl::Other(source_url) => (source_url, false),
        };

        let response = reqwest::get(source_url).await?;
        let bytes = response.bytes().await?;

        if is_model_safetensors {
            let model_safetensors_path = self.model_container.safetensors_path()?;
            let _ = fs::write(&model_safetensors_path, &bytes)?;
            return Ok(());
        }

        let pytorch_path = self.model_container.pytorch_path()?.clone();
        let _ = fs::write(&pytorch_path, &bytes)?;

        pt2safetensors::Pt2Safetensors::default()
            .removes_pt_at_conversion_success()
            .convert(pytorch_path, self.model_container.safetensors_path()?)
            .expect("failed to convert pytorch to safetensors");

        Ok(())
    }

    pub fn safetensors_path(&self) -> anyhow::Result<PathBuf> {
        Ok(self.model_container.safetensors_path()?)
    }
}
