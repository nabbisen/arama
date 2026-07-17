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
        Ok(Self { model_container })
    }

    #[deprecated = "use ModelContainer::download()"]
    pub async fn ensure(&self) -> anyhow::Result<()> {
        self.model_container.download().await
    }

    pub fn safetensors_path(&self) -> anyhow::Result<PathBuf> {
        Ok(self.model_container.safetensors_path()?)
    }
}
