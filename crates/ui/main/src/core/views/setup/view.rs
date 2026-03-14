use iced::{
    Element,
    Length::Fill,
    widget::{button, column, container, progress_bar, row, text},
};

use super::{Setup, message::Message};

impl Setup {
    pub fn view(&self) -> Element<'_, Message> {
        let downloader = self.downloader.view().map(Message::DownloaderMessage);

        let mut download_button = button("Download").padding(10);
        if !self.downloader.is_downloading {
            download_button = download_button.on_press(Message::Download);
        }

        let buttons =
            container(row![download_button, button("Skip").on_press(Message::Skip)].spacing(40))
                .center_x(Fill);

        container(column![downloader, buttons].spacing(40))
            .width(Fill)
            .height(Fill)
            .center(Fill)
            .into()
    }
}
