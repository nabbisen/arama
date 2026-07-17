use std::time::Duration;

use arama_sidecar::media::video::video_engine::{FfmpegDistribution, VideoEngine};
use iced::Task;
use reqwest::header::CONTENT_LENGTH;

use super::{
    Downloader, DownloaderConfig,
    message::Message,
    state::{DownloadProgress, DownloadState},
};

const METADATA_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FfmpegSetupRequest {
    Startup,
    Recheck,
    MissingPair,
    Install,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FfmpegSetupAction {
    ExternalRequired,
    FetchArtifactMetadata,
    Discover,
    Install,
}

fn ffmpeg_setup_action(
    distribution: FfmpegDistribution,
    request: FfmpegSetupRequest,
) -> FfmpegSetupAction {
    match (distribution, request) {
        (_, FfmpegSetupRequest::Startup | FfmpegSetupRequest::Recheck) => {
            FfmpegSetupAction::Discover
        }
        (FfmpegDistribution::External, FfmpegSetupRequest::MissingPair) => {
            FfmpegSetupAction::ExternalRequired
        }
        (FfmpegDistribution::External, FfmpegSetupRequest::Install) => FfmpegSetupAction::Discover,
        (FfmpegDistribution::Managed, FfmpegSetupRequest::MissingPair) => {
            FfmpegSetupAction::FetchArtifactMetadata
        }
        (FfmpegDistribution::Managed, FfmpegSetupRequest::Install) => FfmpegSetupAction::Install,
    }
}

impl Downloader {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::CheckResources => {
                let tasks = self
                    .states
                    .iter()
                    .enumerate()
                    .map(|(id, state)| match &state.config {
                        DownloaderConfig::AiModel(model)
                            if state.download_state != DownloadState::NotRequired =>
                        {
                            let url = model.source_url().download_url();
                            Task::perform(
                                async move { content_length_mib(&url).await },
                                move |size| Message::MetadataChecked(id, size),
                            )
                        }
                        DownloaderConfig::Ffmepg => {
                            if self.distribution == FfmpegDistribution::External {
                                Task::none()
                            } else {
                                start_ffmpeg_task(
                                    id,
                                    self.distribution,
                                    FfmpegSetupRequest::Startup,
                                )
                            }
                        }
                        _ => Task::none(),
                    });
                Task::batch(tasks)
            }
            Message::MetadataChecked(id, size) => {
                self.states[id].file_size = size;
                Task::none()
            }
            Message::FfmpegChecked(id, result) => {
                let state = &mut self.states[id];
                match result {
                    Ok((true, _)) => {
                        state.file_size = None;
                        state.download_state = DownloadState::NotRequired;
                    }
                    Ok((false, size)) => {
                        state.file_size = size;
                        state.download_state = if self.distribution == FfmpegDistribution::External
                        {
                            DownloadState::ExternalRequired
                        } else {
                            DownloadState::Idle
                        };
                    }
                    Err(error) => state.download_state = DownloadState::Errored(error),
                }
                Task::none()
            }
            Message::RecheckFfmpeg(id) => {
                self.states[id].download_state = DownloadState::Checking;
                if self.distribution == FfmpegDistribution::External {
                    Task::done(Message::ExternalFfmpegRequested)
                } else {
                    start_ffmpeg_task(id, self.distribution, FfmpegSetupRequest::Recheck)
                }
            }
            Message::ExternalFfmpegRequested => Task::none(),
            Message::StartDownloads => {
                self.is_downloading = true;

                let tasks = self.states.iter_mut().enumerate().map(|(id, state)| {
                    match state.download_state {
                        DownloadState::Finished
                        | DownloadState::NotRequired
                        | DownloadState::WorkerDraining
                        | DownloadState::ExternalRequired => {
                            return Task::none();
                        }
                        _ => (),
                    }

                    state.download_state = DownloadState::Downloading(0.0);

                    match &state.config {
                        DownloaderConfig::AiModel(model_container) => {
                            let model = model_container.clone();
                            let config = state.config.clone();
                            Task::perform(
                                async move {
                                    match model.download().await {
                                        Ok(()) => DownloadProgress::Finished(config),
                                        Err(error) => DownloadProgress::Errored(error.to_string()),
                                    }
                                },
                                move |progress| Message::AiModelProgressUpdated(id, progress),
                            )
                        }
                        DownloaderConfig::Ffmepg => {
                            match ffmpeg_setup_action(
                                self.distribution,
                                FfmpegSetupRequest::Install,
                            ) {
                                FfmpegSetupAction::Discover => {
                                    state.download_state = DownloadState::Checking;
                                    start_ffmpeg_task(
                                        id,
                                        self.distribution,
                                        FfmpegSetupRequest::Install,
                                    )
                                }
                                FfmpegSetupAction::Install => {
                                    let config = state.config.clone();
                                    Task::perform(
                                        VideoEngine::download_and_install(),
                                        move |result| {
                                            let progress = match result {
                                                Ok(()) => {
                                                    DownloadProgress::Finished(config.clone())
                                                }
                                                Err(error) => {
                                                    DownloadProgress::Errored(error.to_string())
                                                }
                                            };
                                            Message::GeneralProgressUpdated(id, progress)
                                        },
                                    )
                                }
                                FfmpegSetupAction::ExternalRequired
                                | FfmpegSetupAction::FetchArtifactMetadata => unreachable!(
                                    "install requests cannot select a missing-pair action"
                                ),
                            }
                        }
                    }
                });

                Task::batch(tasks)
            }

            Message::AiModelProgressUpdated(id, progress) => {
                match progress {
                    DownloadProgress::Downloading(p) => {
                        self.states[id].download_state = DownloadState::Downloading(p)
                    }
                    DownloadProgress::Finished(_) => {
                        self.states[id].download_state = DownloadState::Finished
                    }
                    DownloadProgress::Errored(err) => {
                        self.states[id].download_state = DownloadState::Errored(err)
                    }
                }

                // Check whether every item has finished or stopped with an error.
                let all_done = self.states.iter().all(|state| {
                    matches!(
                        state.download_state,
                        DownloadState::Finished
                            | DownloadState::Errored(_)
                            | DownloadState::NotRequired
                            | DownloadState::WorkerDraining
                            | DownloadState::ExternalRequired
                    )
                });
                if all_done {
                    self.is_downloading = false;
                }

                Task::none()
            }

            Message::GeneralProgressUpdated(id, progress) => {
                match progress {
                    DownloadProgress::Downloading(p) => {
                        self.states[id].download_state = DownloadState::Downloading(p)
                    }
                    DownloadProgress::Finished(_) => {
                        self.states[id].download_state = DownloadState::Finished
                    }
                    DownloadProgress::Errored(err) => {
                        self.states[id].download_state = DownloadState::Errored(err)
                    }
                }

                // Check whether every item has finished or stopped with an error.
                let all_done = self.states.iter().all(|state| {
                    matches!(
                        state.download_state,
                        DownloadState::Finished
                            | DownloadState::Errored(_)
                            | DownloadState::NotRequired
                            | DownloadState::WorkerDraining
                            | DownloadState::ExternalRequired
                    )
                });
                if all_done {
                    self.is_downloading = false;
                }

                Task::none()
            }
        }
    }
}

fn start_ffmpeg_task(
    id: usize,
    distribution: FfmpegDistribution,
    request: FfmpegSetupRequest,
) -> Task<Message> {
    match ffmpeg_setup_action(distribution, request) {
        FfmpegSetupAction::Discover => check_ffmpeg_task(id, distribution),
        _ => unreachable!("startup, re-check, and external install select discovery"),
    }
}

fn check_ffmpeg_task(id: usize, distribution: FfmpegDistribution) -> Task<Message> {
    Task::perform(
        async move {
            if VideoEngine::discover_toolchain().await.is_some() {
                return Ok((true, None));
            }
            match ffmpeg_setup_action(distribution, FfmpegSetupRequest::MissingPair) {
                FfmpegSetupAction::ExternalRequired => return Ok((false, None)),
                FfmpegSetupAction::FetchArtifactMetadata => {}
                FfmpegSetupAction::Discover | FfmpegSetupAction::Install => {
                    unreachable!("missing-pair requests cannot select a command action")
                }
            }

            let artifact = VideoEngine::download_artifact().map_err(|error| error.to_string())?;
            let client = metadata_client().map_err(|error| error.to_string())?;
            let mut request = client.head(artifact.url);
            if artifact.github_api_asset {
                request = request
                    .header(reqwest::header::ACCEPT, "application/octet-stream")
                    .header(reqwest::header::USER_AGENT, "arama");
            }
            let size = request
                .send()
                .await
                .ok()
                .and_then(|response| response.headers().get(CONTENT_LENGTH).cloned())
                .and_then(|length| length.to_str().ok()?.parse::<u64>().ok())
                .map(|bytes| bytes / 1024 / 1024);
            Ok((false, size))
        },
        move |result| Message::FfmpegChecked(id, result),
    )
}

async fn content_length_mib(url: &str) -> Option<u64> {
    metadata_client()
        .ok()?
        .head(url)
        .send()
        .await
        .ok()?
        .headers()
        .get(CONTENT_LENGTH)?
        .to_str()
        .ok()?
        .parse::<u64>()
        .ok()
        .map(|bytes| bytes / 1024 / 1024)
}

fn metadata_client() -> Result<reqwest::Client, reqwest::Error> {
    reqwest::Client::builder().timeout(METADATA_TIMEOUT).build()
}

#[cfg(test)]
mod policy_tests {
    use arama_sidecar::media::video::video_engine::FfmpegDistribution;

    use super::{FfmpegSetupAction, FfmpegSetupRequest, ffmpeg_setup_action};

    #[test]
    fn external_distribution_never_selects_metadata_or_install_effects() {
        let actions = [
            ffmpeg_setup_action(FfmpegDistribution::External, FfmpegSetupRequest::Startup),
            ffmpeg_setup_action(FfmpegDistribution::External, FfmpegSetupRequest::Recheck),
            ffmpeg_setup_action(
                FfmpegDistribution::External,
                FfmpegSetupRequest::MissingPair,
            ),
            ffmpeg_setup_action(FfmpegDistribution::External, FfmpegSetupRequest::Install),
        ];

        assert_eq!(
            actions,
            [
                FfmpegSetupAction::Discover,
                FfmpegSetupAction::Discover,
                FfmpegSetupAction::ExternalRequired,
                FfmpegSetupAction::Discover,
            ]
        );
        assert!(!actions.contains(&FfmpegSetupAction::FetchArtifactMetadata));
        assert!(!actions.contains(&FfmpegSetupAction::Install));
    }
}
