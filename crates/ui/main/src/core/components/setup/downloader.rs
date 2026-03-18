pub mod config;
pub mod message;
pub mod state;
mod update;
mod util;
mod view;

use arama_sidecar::media::video::video_engine::{FfmpegStatus, VideoEngine};
use config::DownloaderConfig;
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
                let download_state = match &config {
                    DownloaderConfig::AiModel(model_container) => {
                        let safetensors_path = model_container
                            .safetensors_path()
                            .expect("failed to get safetensors path");
                        if safetensors_path.exists() {
                            DownloadState::Finished
                        } else {
                            DownloadState::default()
                        }
                    }
                    DownloaderConfig::Ffmepg => {
                        if VideoEngine::ready() != FfmpegStatus::NotExists {
                            DownloadState::Finished
                        } else {
                            DownloadState::default()
                        }
                    }
                };

                DownloaderState {
                    config,
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
