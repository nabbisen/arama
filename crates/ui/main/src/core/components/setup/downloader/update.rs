use arama_sidecar::media::video::video_engine::VideoEngine;
use iced::Task;

use super::{
    Downloader, DownloaderConfig,
    message::Message,
    state::{DownloadProgress, DownloadState},
    util::{ai_model_download_stream, general_download_stream},
};

impl Downloader {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::StartDownloads => {
                self.is_downloading = true;

                let tasks = self.states.iter_mut().enumerate().map(|(id, state)| {
                    match state.download_state {
                        DownloadState::Finished | DownloadState::NotRequired => {
                            return Task::none();
                        }
                        _ => (),
                    }

                    state.download_state = DownloadState::Downloading(0.0);

                    match &state.config {
                        DownloaderConfig::AiModel(model_container) => Task::run(
                            ai_model_download_stream(model_container.clone()),
                            move |progress| Message::AiModelProgressUpdated(id, progress),
                        ),
                        DownloaderConfig::Ffmepg => {
                            let artifact = match VideoEngine::download_artifact() {
                                Ok(artifact) => artifact,
                                Err(err) => {
                                    state.download_state = DownloadState::Errored(err.to_string());
                                    return Task::none();
                                }
                            };
                            let download_dest_path = match VideoEngine::download_dest_path() {
                                Ok(path) => path,
                                Err(err) => {
                                    state.download_state = DownloadState::Errored(err.to_string());
                                    return Task::none();
                                }
                            };
                            Task::run(
                                general_download_stream(
                                    artifact.url.to_owned(),
                                    download_dest_path,
                                    artifact.expected_sha256.map(str::to_owned),
                                    artifact.github_api_asset,
                                    state.config.clone(),
                                ),
                                move |progress| Message::GeneralProgressUpdated(id, progress),
                            )
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
                    DownloadProgress::Finished(downloader_config) => {
                        if let DownloaderConfig::Ffmepg = downloader_config
                            && let Err(err) = VideoEngine::unpack_archive()
                        {
                            self.states[id].download_state = DownloadState::Errored(format!(
                                "failed to unpack ffmpeg archive: {err}"
                            ));
                            return Task::none();
                        }

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
