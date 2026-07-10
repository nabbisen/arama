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
                        if model_container.clone().ready().unwrap_or(false) {
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
                            match VideoEngine::download_artifact() {
                                Ok(artifact) => {
                                    let client = reqwest::blocking::Client::new();
                                    let mut request = client.head(artifact.url);
                                    if artifact.github_api_asset {
                                        request = request
                                            .header(
                                                reqwest::header::ACCEPT,
                                                "application/octet-stream",
                                            )
                                            .header(reqwest::header::USER_AGENT, "arama");
                                    }

                                    let file_size = match request.send() {
                                        Ok(x) => {
                                            if let Some(content_length) =
                                                x.headers().get(CONTENT_LENGTH)
                                            {
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
                                Err(err) => (DownloadState::Errored(err.to_string()), None),
                            }
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

#[cfg(test)]
mod tests {
    use arama_ai::model::model_container::{ModelContainer, SourceUrl};

    use super::{
        Downloader,
        config::DownloaderConfig,
        message::Message,
        state::{DownloadProgress, DownloadState, DownloaderState},
    };

    fn test_model_config() -> DownloaderConfig {
        DownloaderConfig::AiModel(ModelContainer {
            name: "test-model".to_owned(),
            source_url: SourceUrl::ModelSafetensors("https://example.invalid/model".to_owned()),
            expected_sha256: "unused",
            config_expected_sha256: None,
        })
    }

    fn downloader_with_states(states: Vec<DownloadState>) -> Downloader {
        Downloader {
            is_downloading: true,
            states: states
                .into_iter()
                .map(|download_state| DownloaderState {
                    config: test_model_config(),
                    file_size: None,
                    download_state,
                })
                .collect(),
        }
    }

    #[test]
    fn ai_progress_error_records_error_and_stops_when_all_done() {
        let mut downloader = downloader_with_states(vec![DownloadState::Downloading(42.0)]);

        let _ = downloader.update(Message::AiModelProgressUpdated(
            0,
            DownloadProgress::Errored("checksum mismatch".to_owned()),
        ));

        assert_eq!(
            downloader.states[0].download_state,
            DownloadState::Errored("checksum mismatch".to_owned())
        );
        assert!(!downloader.is_downloading);
    }

    #[test]
    fn ai_progress_keeps_downloading_until_every_state_is_done() {
        let mut downloader =
            downloader_with_states(vec![DownloadState::Downloading(0.0), DownloadState::Idle]);

        let _ = downloader.update(Message::AiModelProgressUpdated(
            0,
            DownloadProgress::Finished(test_model_config()),
        ));

        assert_eq!(downloader.states[0].download_state, DownloadState::Finished);
        assert!(downloader.is_downloading);
    }

    #[test]
    fn general_progress_error_counts_as_done_with_not_required_states() {
        let mut downloader = downloader_with_states(vec![
            DownloadState::Downloading(12.0),
            DownloadState::NotRequired,
        ]);

        let _ = downloader.update(Message::GeneralProgressUpdated(
            0,
            DownloadProgress::Errored("download failed".to_owned()),
        ));

        assert_eq!(
            downloader.states[0].download_state,
            DownloadState::Errored("download failed".to_owned())
        );
        assert!(!downloader.is_downloading);
    }
}
