use std::time::Duration;

use iced::Task;
use reqwest::header::CONTENT_LENGTH;

use super::{
    Downloader, DownloaderConfig,
    message::Message,
    state::{DownloadProgress, DownloadState},
};

const METADATA_TIMEOUT: Duration = Duration::from_secs(5);

impl Downloader {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::CheckResources => {
                let tasks = self.states.iter().enumerate().filter_map(|(id, state)| {
                    let DownloaderConfig::AiModel(model) = &state.config else {
                        return None;
                    };
                    (state.download_state != DownloadState::NotRequired).then(|| {
                        let url = model.source_url().download_url();
                        Task::perform(async move { content_length_mib(&url).await }, move |size| {
                            Message::MetadataChecked(id, size)
                        })
                    })
                });
                Task::batch(tasks)
            }
            Message::MetadataChecked(id, size) => {
                self.states[id].file_size = size;
                Task::none()
            }
            Message::RecheckFfmpeg(id) => {
                self.states[id].download_state = DownloadState::Checking;
                Task::done(Message::ExternalFfmpegRequested)
            }
            Message::ExternalFfmpegRequested => Task::none(),
            Message::StartDownloads => {
                self.is_downloading = true;
                let tasks = self.states.iter_mut().enumerate().map(|(id, state)| {
                    if matches!(
                        state.download_state,
                        DownloadState::Finished
                            | DownloadState::NotRequired
                            | DownloadState::WorkerDraining
                            | DownloadState::ExternalRequired
                    ) {
                        return Task::none();
                    }

                    match &state.config {
                        DownloaderConfig::AiModel(model_container) => {
                            state.download_state = DownloadState::Downloading(0.0);
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
                            state.download_state = DownloadState::Checking;
                            Task::done(Message::ExternalFfmpegRequested)
                        }
                    }
                });
                Task::batch(tasks)
            }
            Message::AiModelProgressUpdated(id, progress) => {
                self.states[id].download_state = match progress {
                    DownloadProgress::Downloading(percent) => DownloadState::Downloading(percent),
                    DownloadProgress::Finished(_) => DownloadState::Finished,
                    DownloadProgress::Errored(error) => DownloadState::Errored(error),
                };
                if self.states.iter().all(|state| {
                    matches!(
                        state.download_state,
                        DownloadState::Finished
                            | DownloadState::Errored(_)
                            | DownloadState::NotRequired
                            | DownloadState::WorkerDraining
                            | DownloadState::ExternalRequired
                    )
                }) {
                    self.is_downloading = false;
                }
                Task::none()
            }
        }
    }
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
