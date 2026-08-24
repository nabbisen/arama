use std::time::Duration;

use arama_ai::model::model_container::DownloadProgress as ModelDownloadProgress;
use iced::Task;
use reqwest::header::CONTENT_LENGTH;
use tokio_stream::wrappers::WatchStream;

use super::{
    Downloader, DownloaderConfig,
    message::Message,
    state::{DownloadBytes, DownloadProgress, DownloadState},
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
            // No local state change here: unlike a re-check, a selection's
            // outcome is unknown until the native picker resolves (it may
            // be cancelled), and the shared authority publish path already
            // updates this item's state once validation actually starts
            // (see `App::publish_ffmpeg_checking`/`publish_ffmpeg_authority`
            // via `Setup::set_ffmpeg_checking`/`set_ffmpeg_ready`).
            Message::SelectFfmpegDirectory => Task::done(Message::FfmpegDirectorySelectRequested),
            Message::FfmpegDirectorySelectRequested => Task::none(),
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
                            state.download_state =
                                DownloadState::Downloading(DownloadBytes::default());
                            let model = model_container.clone();
                            let config = state.config.clone();

                            // Task 036: `download_with_progress` is a
                            // second entry point beside `download`
                            // (untouched, still called by `ensure` and
                            // every other of its 21 sites) - it returns
                            // a live progress receiver alongside the
                            // same future `download` itself awaits, so
                            // both pieces observe the exact same
                            // generation. The one new failure mode this
                            // introduces relative to `download` (a
                            // synchronous `Result` instead of an
                            // always-`Ok` call) is the same identity-
                            // conflict error `download` would otherwise
                            // have surfaced asynchronously - reported
                            // here immediately instead.
                            let Ok((progress_rx, download)) = model.download_with_progress() else {
                                return Task::perform(
                                    async {
                                        DownloadProgress::Errored(
                                            "model name already registered with a different \
                                             specification"
                                                .to_owned(),
                                        )
                                    },
                                    move |progress| Message::AiModelProgressUpdated(id, progress),
                                );
                            };

                            // A joiner's stream starts at whatever the
                            // generation's current progress already is
                            // (`watch::Receiver` semantics), never 0 -
                            // if another download for the same model is
                            // already partway through, this reflects
                            // that immediately rather than restarting
                            // the bar.
                            let progress_task = Task::run(
                                WatchStream::new(progress_rx),
                                move |p: ModelDownloadProgress| {
                                    Message::AiModelProgressUpdated(
                                        id,
                                        DownloadProgress::Downloading(DownloadBytes {
                                            downloaded: p.downloaded,
                                            total: p.total,
                                        }),
                                    )
                                },
                            );
                            let result_task = Task::perform(
                                async move {
                                    match download.await {
                                        Ok(()) => DownloadProgress::Finished(config),
                                        Err(error) => DownloadProgress::Errored(error.to_string()),
                                    }
                                },
                                move |progress| Message::AiModelProgressUpdated(id, progress),
                            );
                            Task::batch([progress_task, result_task])
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
                // Task 036: the progress stream and the final-result
                // future are two independently-scheduled tasks racing
                // on the same generation (batched above) - the
                // generation's `watch::Sender` is only dropped shortly
                // *after* the result is sent, so one last stray
                // `Downloading` message can arrive after `Finished`/
                // `Errored` already did. Once terminal, stay terminal:
                // a state going back to "Downloading 100%" after
                // showing "Ready" would be exactly the kind of
                // regressed-looking number this task exists to remove.
                let already_terminal = matches!(
                    self.states[id].download_state,
                    DownloadState::Finished | DownloadState::Errored(_)
                );
                if already_terminal && matches!(progress, DownloadProgress::Downloading(_)) {
                    return Task::none();
                }
                self.states[id].download_state = match progress {
                    DownloadProgress::Downloading(bytes) => DownloadState::Downloading(bytes),
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
