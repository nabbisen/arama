use iced::{
    Element,
    Length::Fill,
    widget::{container, mouse_area, row, text},
};

use super::{Footer, message::Message};

impl Footer {
    pub fn view(&self) -> Element<'_, Message> {
        // todo
        container(
            row![
                text("Repository:").style(text::secondary),
                mouse_area(text(super::REPOSITORY_URL).style(text::secondary))
                    .on_press(Message::RepositoryLinkClicked(
                        super::REPOSITORY_URL.to_owned(),
                    ))
                    .interaction(iced::mouse::Interaction::Pointer)
            ]
            .spacing(10),
        )
        .padding([0, 20])
        .align_right(Fill)
        .into()
    }
}
