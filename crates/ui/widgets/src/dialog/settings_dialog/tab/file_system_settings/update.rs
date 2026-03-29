use std::fs::remove_dir_all;

use arama_env::cache_dir;
use iced::Task;

use super::{FileSystemSettings, message::Message};

impl FileSystemSettings {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::CacheDelete => {
                let path = cache_dir().unwrap();
                match remove_dir_all(&path) {
                    Ok(_) => (),
                    // todo: error handling
                    Err(err) => eprintln!("{}", err),
                }
            }
        }
        Task::none()
    }
}
