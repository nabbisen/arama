pub mod model;

use std::path::PathBuf;

use model::Model;

const MODEL_DIR: &str = "model";
const SAFETENSORS_MODEL: &str = "model.safetensors";
const PYTORCH_MODEL: &str = "pytorch_model.bin";

pub struct ModelManager {
    model: Model,
}

impl ModelManager {
    pub fn new(model: Model) -> anyhow::Result<Self> {
        let _ = &model.validate_dir()?;

        Ok(Self { model })
    }

    pub async fn get_safetensors_from_pytorch(&self) -> anyhow::Result<()> {
        let response = reqwest::get(&self.model.source_url).await?;
        let bytes = response.bytes().await?;

        let pytorch_path = self.model.pytorch_path()?.clone();

        let _ = tokio::fs::write(&pytorch_path, &bytes).await?;

        pt2safetensors::Pt2Safetensors::default()
            .removes_pt_at_conversion_success()
            .convert(pytorch_path, self.model.safetensors_path()?)
            .expect("failed to convert pytorch to safetensors");

        Ok(())
    }

    pub fn safetensors_path(&self) -> anyhow::Result<PathBuf> {
        Ok(self.model.safetensors_path()?)
    }
}
