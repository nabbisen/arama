use iced::{
    Element,
    Length::Fill,
    widget::{button, column, container, progress_bar, row, text},
};

use super::{Setup, message::Message};

impl Setup {
    pub fn view(&self) -> Element<'_, Message> {
        let clip = row![
            container(text("test")).width(120),
            progress_bar(0.0..=1.0, 0.5)
        ]
        .padding(20);
        let wav2vec2 = row![
            container(text("test")).width(120),
            progress_bar(0.0..=1.0, 0.5)
        ]
        .padding(20);
        let ffmpeg = row![
            container(text("test")).width(120),
            progress_bar(0.0..=1.0, 0.5)
        ]
        .padding(20);
        let subjects = column![clip, wav2vec2, ffmpeg];

        let buttons = container(
            row![
                button("Download").on_press(Message::Download),
                button("Skip").on_press(Message::Skip)
            ]
            .spacing(40),
        )
        .center_x(Fill);

        container(column![subjects, buttons].spacing(40))
            .width(Fill)
            .height(Fill)
            .center(Fill)
            .into()
    }
}
