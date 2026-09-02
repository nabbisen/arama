use arama_env::{cache_dir, local_dir};
use arama_i18n::t;
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

        let fs_info = match DiskSpace::new(&path) {
            Ok(disk_space) => {
                let disk_space = disk_space.as_gb();
                row![
                    text(format!("{:.1} GB", disk_space.available)),
                    text("/"),
                    text(format!("{:.1} GB", disk_space.total)),
                ]
            }
            Err(err) => row![text(format!(
                "{}: {err}",
                t("settings.fs.disk_unavailable")
            ))],
        };

        let cache_exists = cache_dir().is_ok_and(|path| path.exists());
        let button = button(text(t("settings.fs.cache_delete")))
            .style(arama_theme::danger)
            .on_press_maybe(if cache_exists {
                Some(Message::CacheDeleteRequested)
            } else {
                None
            });

        column![fs_info, button].into()
    }
}
