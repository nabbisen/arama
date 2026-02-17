use iced::Element;
use iced::widget::{button, row};

use super::{Menus, message::Message};

impl Menus {
    pub fn view(&self) -> Element<'_, Message> {
        let menus = row![
            button("+").on_press(Message::ScaleUp),
            button("-").on_press(Message::ScaleDown),
            button("x").on_press(Message::Quit),
        ];

        menus.into()
    }
}
