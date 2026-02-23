use arama_env::{DiskSpace, local_dir};
use iced::Element;
use iced::widget::{row, text};

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

        // todo
        row![
            text(format!("{} GB", disk_space.available)),
            text("/"),
            text(format!("{} GB", disk_space.total)),
        ]
        .into()
    }
}
