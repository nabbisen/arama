use std::path::PathBuf;

use candle_core::Device;

use super::model_container::ModelContainer;

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

    pub async fn get_safetensors_from_pytorch(&self) -> anyhow::Result<()> {
        let response = reqwest::get(&self.model_container.source_url).await?;
        let bytes = response.bytes().await?;

        let pytorch_path = self.model_container.pytorch_path()?.clone();

        let _ = tokio::fs::write(&pytorch_path, &bytes).await?;

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
