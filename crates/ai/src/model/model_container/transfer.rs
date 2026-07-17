use std::{fmt::Write as _, path::Path};

use anyhow::{Context, bail};
use arama_env::validate_dir;
use sha2::{Digest, Sha256};
use tokio::{fs, io::AsyncWriteExt};

pub(super) async fn download_authenticated_file(
    client: &reqwest::Client,
    url: &str,
    destination: &Path,
    expected_sha256: &str,
    max_bytes: u64,
) -> anyhow::Result<()> {
    let parent = destination
        .parent()
        .with_context(|| format!("model path has no parent: {}", destination.display()))?;
    validate_dir(parent)?;

    let response = client
        .get(url)
        .send()
        .await
        .with_context(|| format!("failed to fetch {url}"))?;
    if !response.status().is_success() {
        bail!("HTTP error {}: {url}", response.status());
    }

    if response
        .content_length()
        .is_some_and(|length| length > max_bytes)
    {
        bail!("model response exceeds trusted size bound");
    }
    let mut file = fs::File::create(destination)
        .await
        .with_context(|| format!("failed to create {}", destination.display()))?;
    let mut response = response;
    let mut downloaded = 0_u64;
    let mut hasher = Sha256::new();

    let transfer_result: anyhow::Result<()> = async {
        while let Some(chunk) = response
            .chunk()
            .await
            .context("failed to read model body")?
        {
            file.write_all(&chunk)
                .await
                .context("failed to write model")?;
            hasher.update(&chunk);
            downloaded = downloaded.saturating_add(chunk.len() as u64);
            if downloaded > max_bytes {
                bail!("model response exceeds trusted size bound");
            }
        }
        file.flush().await.context("failed to flush model")?;
        Ok(())
    }
    .await;
    drop(file);

    if let Err(error) = transfer_result {
        let _ = fs::remove_file(destination).await;
        return Err(error);
    }

    let actual = sha256_hex(&hasher.finalize());
    if actual != expected_sha256 {
        let _ = fs::remove_file(destination).await;
        bail!("checksum mismatch: expected SHA-256 {expected_sha256}, got {actual}");
    }
    Ok(())
}

pub(super) fn sha256_hex(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut output, "{byte:02x}").expect("writing to String cannot fail");
    }
    output
}
