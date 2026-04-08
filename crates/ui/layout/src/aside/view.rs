use iced::{
    Element, Length,
    widget::{button, column, container, rule},
};
use lucide_icons::iced::{icon_panel_left_close, icon_panel_left_open};

use super::{
    Aside,
    message::{Internal, Message},
};

impl Aside {
    pub fn view(&self) -> Element<'_, Message> {
        if !self.is_open {
            return container(
                button(icon_panel_left_open())
                    .style(button::background)
                    .on_press(Message::Internal(Internal::Open)),
            )
            .align_left(44)
            .into();
        }

        let close_button = container(
            button(icon_panel_left_close())
                .style(button::background)
                .on_press(Message::Internal(Internal::Close)),
        );

        let rule = rule::horizontal(1).style(|theme| rule::Style {
            fill_mode: rule::FillMode::Padded(10),
            ..rule::default(theme)
        });

        let dir_tree = container(
            self.dir_tree
                .view()
                .map(|x| Message::Internal(Internal::DirTreeMessage(x))),
        )
        .padding([5, 0]);

        column![close_button, rule, dir_tree]
            .width(Length::Shrink)
            .spacing(5)
            .into()
    }
}
