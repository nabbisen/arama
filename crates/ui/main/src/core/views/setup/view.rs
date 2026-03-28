use arama_env::{DiskSpace, MIN_SETUP_DISKSPACE_MB, local_dir};
use iced::{
    Alignment::{self, Center},
    Element,
    Length::Fill,
    widget::{button, column, container, row, text},
};

use super::{Setup, message::Message};

impl Setup {
    pub fn view(&self) -> Element<'_, Message> {
        let downloader = self.downloader.view().map(Message::DownloaderMessage);

        let local_dir = local_dir().unwrap();
        let disk_space = DiskSpace::new(&local_dir).expect("failed to get file system info ");
        let disk_space_ok = (MIN_SETUP_DISKSPACE_MB as f64) < disk_space.as_mb().available;

        let download_button = button("Download").padding(10).on_press_maybe(
            if disk_space_ok && !self.downloader.is_downloading {
                Some(Message::Download)
            } else {
                None
            },
        );

        let buttons = container(
            row![
                download_button,
                button("Skip").style(button::secondary).on_press_maybe(
                    if !self.downloader.is_downloading {
                        Some(Message::Skip)
                    } else {
                        None
                    }
                )
            ]
            .align_y(Alignment::Center)
            .spacing(40)
            .padding([10, 0]),
        );

        let mut content = column![downloader].align_x(Center).spacing(20);
        if !disk_space_ok {
            content = content.push(text("No enough space on device for downloader."))
        }
        content = content.push(buttons);

        container(content)
            .width(Fill)
            .height(Fill)
            .center(Fill)
            .into()
    }
}
