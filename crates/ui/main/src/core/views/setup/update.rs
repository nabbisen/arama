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
                let task = self
                    .downloader
                    .update(message)
                    .map(Message::DownloaderMessage);
                if !self.downloader.is_downloading {
                    self.finished = true;
                }
                task
            }
        }
    }
}
