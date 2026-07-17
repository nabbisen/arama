use iced::Task;

use crate::components::setup::downloader;

use super::{Setup, message::Message};

impl Setup {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Download => self
                .downloader
                .update(downloader::message::Message::StartDownloads)
                .map(Message::DownloaderMessage),
            Message::Skip => {
                self.finished = true;
                Task::none()
            }
            Message::DownloaderMessage(message) => {
                if matches!(
                    message,
                    downloader::message::Message::ExternalFfmpegRequested
                ) {
                    return Task::done(Message::FfmpegRecheckRequested);
                }
                let download_progress = matches!(
                    message,
                    downloader::message::Message::AiModelProgressUpdated(_, _)
                        | downloader::message::Message::GeneralProgressUpdated(_, _)
                );
                let task = self
                    .downloader
                    .update(message)
                    .map(Message::DownloaderMessage);
                self.ready = self.downloader.requirements_ready();
                if download_progress && self.ready {
                    self.finished = true;
                }
                task
            }
            Message::FfmpegRecheckRequested => Task::none(),
        }
    }
}
