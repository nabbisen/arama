use iced::Element;
use iced::widget::{button, row};

use super::SettingsNav;
use super::message::Message;

impl SettingsNav {
    pub fn view(&self) -> Element<'_, Message> {
        // todo
        row![button("⚙️").on_press(Message::SettingsClick)].into()
    }
}
