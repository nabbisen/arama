use iced::{
    Element,
    widget::container,
    widget::{mouse_area, row, text},
};

use super::{About, message::Message};

impl About {
    pub fn view(&self) -> Element<'_, Message> {
        container(
            row![
                text("Repository:"),
                mouse_area(text(super::REPOSITORY_URL))
                    .on_press(Message::RepositoryLinkClicked(
                        super::REPOSITORY_URL.to_owned(),
                    ))
                    .interaction(iced::mouse::Interaction::Pointer)
            ]
            .spacing(10),
        )
        .padding([0, 20])
        .into()
    }
}
