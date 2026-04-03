pub mod config;
pub mod message;
pub mod state;
mod update;
mod util;
mod view;

use arama_sidecar::media::video::video_engine::{FfmpegStatus, VideoEngine};
use config::DownloaderConfig;
use reqwest::header::CONTENT_LENGTH;
use state::{DownloadState, DownloaderState};

#[derive(Debug, Clone)]
pub struct Downloader {
    pub is_downloading: bool,
    states: Vec<DownloaderState>,
}

impl Downloader {
    pub fn new(configs: Vec<DownloaderConfig>) -> Self {
        let states = configs
            .into_iter()
            .map(|config| {
                let (download_state, file_size) = match &config {
                    DownloaderConfig::AiModel(model_container) => {
                        let safetensors_path = model_container
                            .safetensors_path()
                            .expect("failed to get safetensors path");
                        if safetensors_path.exists() {
                            (DownloadState::NotRequired, None)
                        } else {
                            let file_size = match reqwest::blocking::Client::new()
                                .head(model_container.source_url.download_url())
                                .send()
                            {
                                Ok(x) => {
                                    if let Some(content_length) = x.headers().get(CONTENT_LENGTH) {
                                        if let Ok(x) = content_length
                                            .to_str()
                                            .unwrap_or_default()
                                            .parse::<u64>()
                                        {
                                            Some(x / 1024 / 1024)
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    }
                                }
                                Err(_) => None,
                            };

                            (DownloadState::default(), file_size)
                        }
                    }
                    DownloaderConfig::Ffmepg => {
                        if VideoEngine::ready() != FfmpegStatus::NotExists {
                            (DownloadState::NotRequired, None)
                        } else {
                            let file_size = match reqwest::blocking::Client::new()
                                .head(VideoEngine::download_url().unwrap())
                                .send()
                            {
                                Ok(x) => {
                                    if let Some(content_length) = x.headers().get(CONTENT_LENGTH) {
                                        if let Ok(x) = content_length
                                            .to_str()
                                            .unwrap_or_default()
                                            .parse::<u64>()
                                        {
                                            Some(x / 1024 / 1024)
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    }
                                }
                                Err(_) => None,
                            };

                            (DownloadState::default(), file_size)
                        }
                    }
                };

                DownloaderState {
                    config,
                    file_size,
                    download_state,
                }
            })
            .collect();

        Self {
            is_downloading: false,
            states,
        }
    }
}
