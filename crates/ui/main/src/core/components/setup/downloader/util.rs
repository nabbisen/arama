//! Download stream helpers for the setup downloader.
//!
//! ## Why `try_send` for progress updates
//!
//! The iced event loop processes messages at frame rate (~60 fps).  If the
//! download server sends data in small chunks — which ffmpeg hosts do — calling
//! `output.send(progress).await` on every chunk fills the channel (capacity 100)
//! and stalls the download waiting for the UI to drain it.  HuggingFace CDN
//! returns large chunks, so this throttle is rarely hit there; ffmpeg hosts hit
//! it constantly, making the download appear much slower.
//!
//! `try_send` is non-blocking: it delivers the progress update when the channel
//! has space and silently drops it when full.  The download itself never waits
//! on the UI.  Progress display remains smooth because updates come in at least
//! as fast as the UI can display them.

use std::{
    fmt::Write as _,
    path::{Path, PathBuf},
};

use arama_ai::model::model_container::{ModelContainer, SourceUrl};
use arama_env::validate_dir;
use iced::futures::{SinkExt, StreamExt, channel::mpsc::Sender};
use sha2::{Digest, Sha256};
use tokio::fs::{self, File};
use tokio::io::{AsyncWriteExt, BufWriter};

use super::{config::DownloaderConfig, state::DownloadProgress};

/// Write buffer size: 256 KB.  The default `BufWriter::new` uses 8 KB, causing
/// frequent small write syscalls on fast connections.
const WRITE_BUF_CAPACITY: usize = 256 * 1024;

// ---------------------------------------------------------------------------
// Shared streaming core
// ---------------------------------------------------------------------------

/// Stream `response` to `dest`, via a `.part` temporary file that is renamed
/// to `dest` on success.  Reports byte progress through `output` using
/// non-blocking `try_send` so the download is never throttled by UI rendering.
///
/// Returns an error description on failure; the `.part` file is removed before
/// returning so no partial files are left behind.
async fn stream_to_file(
    response: reqwest::Response,
    dest: &Path,
    expected_sha256: Option<&str>,
    output: &mut Sender<DownloadProgress>,
) -> Result<(), String> {
    let total = response.content_length().unwrap_or(0) as f32;
    let mut downloaded = 0.0f32;
    let mut hasher = Sha256::new();

    // Ensure the parent directory exists.
    let parent = dest
        .parent()
        .ok_or_else(|| format!("no parent directory: {}", dest.display()))?;
    validate_dir(parent).map_err(|e| format!("could not create parent directory: {e}"))?;

    // Write to a `.part` file; rename to the final name only on success.
    let part = format!("{}.part", dest.to_string_lossy());

    let file = File::create(&part)
        .await
        .map_err(|e| format!("could not create download file: {e}"))?;
    let mut writer = BufWriter::with_capacity(WRITE_BUF_CAPACITY, file);
    let mut stream = response.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = match item {
            Ok(c) => c,
            Err(e) => {
                let _ = fs::remove_file(&part).await;
                return Err(format!("connection interrupted: {e}"));
            }
        };

        if let Err(e) = writer.write_all(&chunk).await {
            let _ = fs::remove_file(&part).await;
            return Err(format!("write error: {e}"));
        }

        hasher.update(&chunk);
        downloaded += chunk.len() as f32;
        let pct = if total > 0.0 {
            (downloaded / total) * 100.0
        } else {
            0.0
        };
        // Non-blocking: dropped silently when the channel is full.
        // The download must never stall waiting for the UI.
        let _ = output.try_send(DownloadProgress::Downloading(pct));
    }

    if let Err(e) = writer.flush().await {
        let _ = fs::remove_file(&part).await;
        return Err(format!("flush error: {e}"));
    }
    drop(writer); // release file handle before rename

    if let Some(expected_sha256) = expected_sha256 {
        let digest = hasher.finalize();
        let actual = sha256_hex(&digest);
        if actual != expected_sha256 {
            let _ = fs::remove_file(&part).await;
            return Err(format!(
                "checksum mismatch: expected SHA-256 {expected_sha256}, got {actual}"
            ));
        }
    }

    if let Err(e) = fs::rename(&part, dest).await {
        let _ = fs::remove_file(&part).await;
        return Err(format!("rename error: {e}"));
    }

    Ok(())
}

/// Fetch `url`, check for HTTP success, and return the response.
async fn fetch(url: &str, github_api_asset: bool) -> Result<reqwest::Response, String> {
    let client = reqwest::Client::new();
    let mut request = client.get(url);
    if github_api_asset {
        request = request
            .header(reqwest::header::ACCEPT, "application/octet-stream")
            .header(reqwest::header::USER_AGENT, "arama");
    }

    let response = request
        .send()
        .await
        .map_err(|e| format!("network error: {e}"))?;
    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()));
    }
    Ok(response)
}

// ---------------------------------------------------------------------------
// Public stream constructors
// ---------------------------------------------------------------------------

/// General-purpose download stream (used for ffmpeg).
///
/// Downloads `url` to `download_dest_path` and emits [`DownloadProgress`]
/// items.  Uses non-blocking progress sends so the transfer speed is never
/// limited by the iced event loop.
pub fn general_download_stream(
    url: String,
    download_dest_path: PathBuf,
    expected_sha256: Option<String>,
    github_api_asset: bool,
    downloader_config: DownloaderConfig,
) -> impl StreamExt<Item = DownloadProgress> {
    iced::stream::channel(
        100,
        move |mut output: Sender<DownloadProgress>| async move {
            if download_dest_path.exists() {
                let _ = output
                    .send(DownloadProgress::Errored("file already exists".to_string()))
                    .await;
                return;
            }

            let response = match fetch(&url, github_api_asset).await {
                Ok(r) => r,
                Err(e) => {
                    let _ = output.send(DownloadProgress::Errored(e)).await;
                    return;
                }
            };

            if let Err(e) = stream_to_file(
                response,
                &download_dest_path,
                expected_sha256.as_deref(),
                &mut output,
            )
            .await
            {
                let _ = output.send(DownloadProgress::Errored(e)).await;
                return;
            }

            let _ = output
                .send(DownloadProgress::Finished(downloader_config))
                .await;
        },
    )
}

/// AI-model download stream (HuggingFace safetensors + optional config JSON).
///
/// Downloads the model weights and, when present, an accompanying config
/// JSON file.  Uses non-blocking progress sends for the weights transfer.
pub fn ai_model_download_stream(
    model_container: ModelContainer,
) -> impl StreamExt<Item = DownloadProgress> {
    iced::stream::channel(
        100,
        move |mut output: Sender<DownloadProgress>| async move {
            let safetensors_path = model_container
                .safetensors_path()
                .expect("failed to get safetensors path");

            if model_container.clone().ready().unwrap_or(false) {
                let _ = output
                    .send(DownloadProgress::Errored("file already exists".to_string()))
                    .await;
                return;
            }

            if safetensors_path.exists() {
                cleanup_primary_artifact(&safetensors_path).await;
            }

            // Resolve the primary download URL and save path.
            let (model_url, path_to_save) = match &model_container.source_url {
                SourceUrl::ModelSafetensors(u) | SourceUrl::ModelSafetensorsConfigJson((u, _)) => (
                    u.clone(),
                    model_container
                        .safetensors_path()
                        .expect("failed to get safetensors path"),
                ),
                SourceUrl::PyTorch(u) => (
                    u.clone(),
                    model_container
                        .pytorch_path()
                        .expect("failed to get pytorch path"),
                ),
            };

            let response = match fetch(&model_url, false).await {
                Ok(r) => r,
                Err(e) => {
                    let _ = output.send(DownloadProgress::Errored(e)).await;
                    return;
                }
            };

            if let Err(e) = stream_to_file(
                response,
                &path_to_save,
                Some(model_container.expected_sha256),
                &mut output,
            )
            .await
            {
                let _ = output.send(DownloadProgress::Errored(e)).await;
                return;
            }

            if let Err(e) = model_container.ensure_safetensors() {
                let _ = output
                    .send(DownloadProgress::Errored(format!(
                        "model conversion error: {e}"
                    )))
                    .await;
                return;
            }

            // Optional small config JSON (downloaded in full, no progress needed).
            if let SourceUrl::ModelSafetensorsConfigJson((_, config_url)) =
                &model_container.source_url
            {
                let parent = match path_to_save.parent() {
                    Some(parent) => parent,
                    None => {
                        cleanup_primary_artifact(&path_to_save).await;
                        let _ = output
                            .send(DownloadProgress::Errored(format!(
                                "model path has no parent directory: {}",
                                path_to_save.display()
                            )))
                            .await;
                        return;
                    }
                };

                let res = match fetch(config_url, false).await {
                    Ok(r) => r,
                    Err(e) => {
                        cleanup_primary_artifact(&path_to_save).await;
                        let _ = output.send(DownloadProgress::Errored(e)).await;
                        return;
                    }
                };

                let bytes = match res.bytes().await {
                    Ok(b) => b,
                    Err(e) => {
                        cleanup_primary_artifact(&path_to_save).await;
                        let _ = output
                            .send(DownloadProgress::Errored(format!(
                                "config download error: {e}"
                            )))
                            .await;
                        return;
                    }
                };

                if let Some(expected_sha256) = model_container.config_expected_sha256 {
                    let digest = Sha256::digest(&bytes);
                    let actual = sha256_hex(&digest);
                    if actual != expected_sha256 {
                        cleanup_primary_artifact(&path_to_save).await;
                        let _ = output
                            .send(DownloadProgress::Errored(format!(
                                "config checksum mismatch: expected SHA-256 {expected_sha256}, got {actual}"
                            )))
                            .await;
                        return;
                    }
                }

                let url = match reqwest::Url::parse(config_url) {
                    Ok(url) => url,
                    Err(e) => {
                        cleanup_primary_artifact(&path_to_save).await;
                        let _ = output
                            .send(DownloadProgress::Errored(format!(
                                "config URL parse error: {e}"
                            )))
                            .await;
                        return;
                    }
                };
                let filename = url
                    .path_segments()
                    .and_then(|mut s| s.next_back())
                    .filter(|s| !s.is_empty())
                    .unwrap_or("model.bin");

                let config_path = parent.join(filename);
                if let Err(e) = fs::write(&config_path, bytes).await {
                    cleanup_primary_artifact(&path_to_save).await;
                    let _ = output
                        .send(DownloadProgress::Errored(format!("config save error: {e}")))
                        .await;
                    return;
                }
            }

            let _ = output
                .send(DownloadProgress::Finished(DownloaderConfig::AiModel(
                    model_container,
                )))
                .await;
        },
    )
}

async fn cleanup_primary_artifact(path: &Path) {
    let _ = fs::remove_file(path).await;
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hex = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut hex, "{byte:02x}").expect("writing to string cannot fail");
    }
    hex
}

#[cfg(test)]
mod tests {
    use sha2::{Digest, Sha256};

    use super::sha256_hex;

    #[test]
    fn sha256_hex_formats_digest() {
        let digest = Sha256::digest(b"arama");
        assert_eq!(
            sha256_hex(&digest),
            "0d22554a4efcf5eb5aa3bef02fa51ce1a1c8ba77fe45d6d959148250c1211702"
        );
    }
}
