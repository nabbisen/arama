use iced::{
    Alignment, Element,
    Length::Fill,
    widget::{button, column, container, row},
};

use super::{Setup, message::Message};

impl Setup {
    pub fn view(&self) -> Element<'_, Message> {
        let downloader = self.downloader.view().map(Message::DownloaderMessage);

        let download_button =
            button("Download")
                .padding(10)
                .on_press_maybe(if !self.downloader.is_downloading {
                    Some(Message::Download)
                } else {
                    None
                });

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
            .spacing(40),
        )
        .center_x(Fill);

        container(column![downloader, buttons].spacing(40))
            .width(Fill)
            .height(Fill)
            .center(Fill)
            .into()
    }
}
