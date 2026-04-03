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
                            let url = VideoEngine::download_url().unwrap();
                            let download_dest_path = VideoEngine::download_dest_path().unwrap();
                            Task::run(
                                general_download_stream(
                                    url,
                                    download_dest_path,
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

                // すべて完了またはエラーで止まったかチェック
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
                        match downloader_config {
                            DownloaderConfig::Ffmepg => VideoEngine::unpack_archive().unwrap(),
                            _ => (),
                        };

                        self.states[id].download_state = DownloadState::Finished
                    }
                    DownloadProgress::Errored(err) => {
                        self.states[id].download_state = DownloadState::Errored(err)
                    }
                }

                // すべて完了またはエラーで止まったかチェック
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
