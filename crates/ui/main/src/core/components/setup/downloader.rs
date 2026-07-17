pub mod config;
pub mod message;
pub mod state;
mod update;
mod view;

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
                        if model_container.clone().ready().unwrap_or(false) {
                            DownloadState::NotRequired
                        } else {
                            DownloadState::default()
                        }
                    }
                    DownloaderConfig::Ffmepg => DownloadState::ExternalRequired,
                };

                DownloaderState {
                    config,
                    file_size: None,
                    download_state,
                }
            })
            .collect();

        Self {
            is_downloading: false,
            states,
        }
    }

    pub fn initial_task(&self) -> iced::Task<message::Message> {
        iced::Task::done(message::Message::CheckResources)
    }

    pub fn requirements_ready(&self) -> bool {
        let ready = |state: &DownloaderState| {
            matches!(
                state.download_state,
                DownloadState::Finished | DownloadState::NotRequired
            )
        };

        let named_model_ready = |name: &str| {
            self.states.iter().any(|state| {
                matches!(&state.config, DownloaderConfig::AiModel(model) if model.name() == name)
                    && ready(state)
            })
        };
        crate::views::setup::util::resources_ready(named_model_ready(
            arama_ai::model::model_container::clip::model().name(),
        ))
    }

    pub fn can_start_downloads(&self) -> bool {
        !self
            .states
            .iter()
            .any(|state| state.download_state == DownloadState::Checking)
    }

    pub fn set_external_ffmpeg_ready(&mut self, ready: bool) {
        if let Some(state) = self
            .states
            .iter_mut()
            .find(|state| matches!(state.config, DownloaderConfig::Ffmepg))
        {
            state.file_size = None;
            state.download_state = if ready {
                DownloadState::NotRequired
            } else {
                DownloadState::ExternalRequired
            };
        }
    }

    pub fn set_external_ffmpeg_checking(&mut self) {
        if let Some(state) = self
            .states
            .iter_mut()
            .find(|state| matches!(state.config, DownloaderConfig::Ffmepg))
        {
            state.download_state = DownloadState::Checking;
        }
    }

    pub fn set_external_ffmpeg_draining(&mut self) {
        if let Some(state) = self
            .states
            .iter_mut()
            .find(|state| matches!(state.config, DownloaderConfig::Ffmepg))
        {
            state.download_state = DownloadState::WorkerDraining;
        }
    }

    #[cfg(test)]
    pub(crate) fn from_states_for_test(
        states: Vec<(DownloaderConfig, DownloadState)>,
        is_downloading: bool,
    ) -> Self {
        Self {
            is_downloading,
            states: states
                .into_iter()
                .map(|(config, download_state)| DownloaderState {
                    config,
                    file_size: None,
                    download_state,
                })
                .collect(),
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
        DownloaderConfig::AiModel(
            ModelContainer::new(
                "test-model",
                SourceUrl::ModelSafetensors("https://example.invalid/model".to_owned()),
                "0000000000000000000000000000000000000000000000000000000000000000",
                None,
                1024,
                None,
            )
            .expect("valid test model"),
        )
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
    fn start_request_transitions_external_ffmpeg_to_discovery() {
        let mut downloader = Downloader::from_states_for_test(
            vec![(DownloaderConfig::Ffmepg, DownloadState::Idle)],
            false,
        );

        let _ = downloader.update(Message::StartDownloads);

        assert_eq!(downloader.states[0].download_state, DownloadState::Checking);
    }

    #[test]
    fn setup_readiness_is_clip_only_with_external_ffmpeg() {
        let clip = DownloaderConfig::AiModel(arama_ai::model::model_container::clip::model());
        let wav2vec2 =
            DownloaderConfig::AiModel(arama_ai::model::model_container::wav2vec2::model());
        let downloader = Downloader::from_states_for_test(
            vec![
                (clip, DownloadState::NotRequired),
                (wav2vec2, DownloadState::Idle),
                (DownloaderConfig::Ffmepg, DownloadState::Idle),
            ],
            false,
        );

        assert!(downloader.requirements_ready());
    }
}
