use arama_env::{MIN_SETUP_DISKSPACE_MB, local_dir};
use arama_i18n::{t, t_with};
use disk_space::DiskSpace;
use iced::{
    Alignment::{self, Center},
    Element,
    Length::Fill,
    widget::{Text, button, column, container, row, text},
};

use super::{Setup, message::Message};

impl Setup {
    pub fn view(&self) -> Element<'_, Message> {
        let downloader = self.downloader.view().map(Message::DownloaderMessage);

        let disk_status = setup_disk_status();

        let download_button = button(text(t("setup.download")))
            .padding(10)
            .on_press_maybe(
                if disk_status.can_download
                    && self.downloader.can_start_downloads()
                    && !self.downloader.is_downloading
                {
                    Some(Message::Download)
                } else {
                    None
                },
            );

        let buttons = container(
            row![
                download_button,
                button(text(t("setup.skip")))
                    .style(arama_theme::secondary)
                    .on_press_maybe(if !self.downloader.is_downloading {
                        Some(Message::Skip)
                    } else {
                        None
                    })
            ]
            .align_y(Alignment::Center)
            .spacing(40)
            .padding([10, 0]),
        );

        let mut content = column![downloader].align_x(Center).spacing(20);
        if let Some(message) = disk_status.message {
            content = content.push(message);
        }
        content = content.push(buttons);

        container(content)
            .width(Fill)
            .height(Fill)
            .center(Fill)
            .into()
    }
}

struct SetupDiskStatus<'a> {
    can_download: bool,
    message: Option<Text<'a>>,
}

fn setup_disk_status<'a>() -> SetupDiskStatus<'a> {
    let local_dir = match local_dir() {
        Ok(local_dir) => local_dir,
        Err(err) => {
            return SetupDiskStatus {
                can_download: false,
                message: Some(text(format!("{}: {}", t("setup.status.error"), err))),
            };
        }
    };

    let disk_space = match DiskSpace::new(&local_dir) {
        Ok(disk_space) => disk_space,
        Err(err) => {
            return SetupDiskStatus {
                can_download: false,
                message: Some(text(format!("{}: {}", t("setup.status.error"), err))),
            };
        }
    };

    let disk_space_ok = (MIN_SETUP_DISKSPACE_MB as f64) < disk_space.as_mb().available;
    SetupDiskStatus {
        can_download: disk_space_ok,
        message: (!disk_space_ok).then(|| {
            text(t_with(
                "setup.no_space",
                &[("{mb}", &MIN_SETUP_DISKSPACE_MB.to_string())],
            ))
        }),
    }
}
