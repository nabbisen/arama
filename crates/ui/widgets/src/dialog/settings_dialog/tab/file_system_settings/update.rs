use std::fs::remove_dir_all;

use arama_env::cache_dir;
use iced::Task;

use super::{FileSystemSettings, message::Message};

impl FileSystemSettings {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::CacheDelete => match cache_dir() {
                Ok(path) => {
                    if let Err(err) = remove_dir_all(&path) {
                        eprintln!("failed to remove cache directory: {err}");
                    }
                }
                Err(err) => eprintln!("failed to get cache directory: {err}"),
            },
        }
        Task::none()
    }
}
