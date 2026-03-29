use arama_env::local_dir;
use disk_space::DiskSpace;
use iced::Element;
use iced::widget::{column, row, text};

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

        // todo
        column![fs_info].into()
    }
}
