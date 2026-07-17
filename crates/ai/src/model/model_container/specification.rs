use anyhow::bail;

use super::{ModelContainer, SourceUrl};

pub(super) fn validate_model_specification(model: &ModelContainer) -> anyhow::Result<()> {
    if model.name.is_empty()
        || !model
            .name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        bail!("model name must contain only ASCII letters, digits, '-' or '_'");
    }
    validate_sha256(model.expected_sha256)?;
    if let Some(digest) = model.config_expected_sha256 {
        validate_sha256(digest)?;
    }
    if model.max_model_bytes == 0 {
        bail!("model has no trusted primary size bound");
    }
    let has_config_url = matches!(&model.source_url, SourceUrl::ModelSafetensorsConfigJson(_));
    let has_config_digest = model.config_expected_sha256.is_some();
    let has_config_limit = model.max_config_bytes.is_some_and(|limit| limit > 0);
    if has_config_url != has_config_digest || has_config_url != has_config_limit {
        bail!("model configuration URL, digest, and size bound must be specified together");
    }
    Ok(())
}

fn validate_sha256(digest: &str) -> anyhow::Result<()> {
    if digest.len() != 64
        || !digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        bail!("model SHA-256 must be 64 lowercase hexadecimal characters");
    }
    Ok(())
}
