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

use std::path::{Path, PathBuf};

use arama_ai::model::model_container::{ModelContainer, SourceUrl};
use arama_env::validate_dir;
use iced::futures::{SinkExt, StreamExt, channel::mpsc::Sender};
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
    output: &mut Sender<DownloadProgress>,
) -> Result<(), String> {
    let total = response.content_length().unwrap_or(0) as f32;
    let mut downloaded = 0.0f32;

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

    if let Err(e) = fs::rename(&part, dest).await {
        let _ = fs::remove_file(&part).await;
        return Err(format!("rename error: {e}"));
    }

    Ok(())
}

/// Fetch `url`, check for HTTP success, and return the response.
async fn fetch(url: &str) -> Result<reqwest::Response, String> {
    let response = reqwest::get(url)
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

            let response = match fetch(&url).await {
                Ok(r) => r,
                Err(e) => {
                    let _ = output.send(DownloadProgress::Errored(e)).await;
                    return;
                }
            };

            if let Err(e) = stream_to_file(response, &download_dest_path, &mut output).await {
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

            if safetensors_path.exists() {
                let _ = output
                    .send(DownloadProgress::Errored("file already exists".to_string()))
                    .await;
                return;
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

            let response = match fetch(&model_url).await {
                Ok(r) => r,
                Err(e) => {
                    let _ = output.send(DownloadProgress::Errored(e)).await;
                    return;
                }
            };

            if let Err(e) = stream_to_file(response, &path_to_save, &mut output).await {
                let _ = output.send(DownloadProgress::Errored(e)).await;
                return;
            }

            let _ = model_container
                .ensure_safetensors()
                .expect("failed to ensure safetensors");

            // Optional small config JSON (downloaded in full, no progress needed).
            if let SourceUrl::ModelSafetensorsConfigJson((_, config_url)) =
                &model_container.source_url
            {
                let parent = path_to_save
                    .parent()
                    .expect("model path has no parent directory");

                let res = match fetch(config_url).await {
                    Ok(r) => r,
                    Err(e) => {
                        let _ = output.send(DownloadProgress::Errored(e)).await;
                        return;
                    }
                };

                let bytes = match res.bytes().await {
                    Ok(b) => b,
                    Err(e) => {
                        let _ = output
                            .send(DownloadProgress::Errored(format!(
                                "config download error: {e}"
                            )))
                            .await;
                        return;
                    }
                };

                let url = reqwest::Url::parse(config_url).unwrap();
                let filename = url
                    .path_segments()
                    .and_then(|s| s.last())
                    .filter(|s| !s.is_empty())
                    .unwrap_or("model.bin");

                let config_path = parent.join(filename);
                if let Err(e) = fs::write(&config_path, bytes).await {
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
