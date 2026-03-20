use arama_sidecar::media::video::video_engine::VideoEngine;
use iced::Task;

use crate::components::setup::downloader::config::DownloaderConfig;

use super::{
    Downloader,
    message::Message,
    state::{DownloadProgress, DownloadState},
    util::download_stream,
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
                        DownloaderConfig::AiModel(model_container) => {
                            // ストリーム関数を呼び出す
                            Task::run(download_stream(model_container.clone()), move |progress| {
                                Message::ProgressUpdated(id, progress)
                            })
                        }
                        DownloaderConfig::Ffmepg => match VideoEngine::download() {
                            Ok(_) => {
                                Task::done(Message::ProgressUpdated(id, DownloadProgress::Finished))
                            }
                            Err(err) => Task::done(Message::ProgressUpdated(
                                id,
                                DownloadProgress::Errored(err.to_string()),
                            )),
                        },
                    }
                });

                Task::batch(tasks)
            }

            Message::ProgressUpdated(id, progress) => {
                match progress {
                    DownloadProgress::Downloading(p) => {
                        self.states[id].download_state = DownloadState::Downloading(p)
                    }
                    DownloadProgress::Finished => {
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
