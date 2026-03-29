use arama_env::{cache_dir, local_dir};
use disk_space::DiskSpace;
use iced::Element;
use iced::widget::{button, column, row, text};

use super::{FileSystemSettings, message::Message};

impl FileSystemSettings {
    pub fn view(&self) -> Element<'_, Message> {
        let path = if let Ok(x) = local_dir() {
            x
        } else {
            ".".into()
        };

        let disk_space = DiskSpace::new(&path)
            .expect("failed to get file system info ")
            .as_gb();

        let fs_info = row![
            text(format!("{:.1} GB", disk_space.available)),
            text("/"),
            text(format!("{:.1} GB", disk_space.total)),
        ];

        let button = button("Cache delete").on_press_maybe(if cache_dir().unwrap().exists() {
            Some(Message::CacheDelete)
        } else {
            None
        });

        column![fs_info, button].into()
    }
}
