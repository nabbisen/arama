use std::path::{Path, PathBuf};

pub fn has_model() -> bool {
    Path::new(crate::SAFETENSORS_MODEL).exists()
}

pub async fn get_model() -> Result<(), String> {
    let response = reqwest::get(crate::URL).await;

    let path = match response {
        Ok(response) => {
            let bytes = response.bytes().await.expect("failed to get model bytes");
            let path = crate::PYTORCH_MODEL;
            match tokio::fs::write(path, &bytes).await {
                Ok(_) => PathBuf::from(path),
                Err(err) => return Err(err.to_string()),
            }
        }
        Err(err) => return Err(err.to_string()),
    };

    pt2safetensors::Pt2Safetensors::default()
        .removes_pt_at_conversion_success()
        .convert(&path, &crate::SAFETENSORS_MODEL.into())
        .expect("failed to convert pytorch to safetensors");

    Ok(())
}
