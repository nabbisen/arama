use iced::{
    Element,
    Length::Fill,
    widget::{container, row, text},
};

use super::{Footer, message::Message};

impl Footer {
    pub fn view(&self) -> Element<'_, Message> {
        let files_label = if 1 < self.files_count {
            "files"
        } else {
            "file"
        };
        let dirs_label = if 1 < self.files_count { "dirs" } else { "dir" };

        container(
            row![
                text(format!("{} {}", self.files_count, files_label)).style(text::secondary),
                text(format!("({} {} scanned)", self.dirs_count, dirs_label))
                    .style(text::secondary),
            ]
            .spacing(10),
        )
        .padding([10, 20])
        .align_right(Fill)
        .into()
    }
}
