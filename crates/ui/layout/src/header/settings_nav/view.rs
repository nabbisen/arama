use iced::Element;
use iced::widget::{button, row};
use lucide_icons::iced::icon_settings;

use super::SettingsNav;
use super::message::Message;

impl SettingsNav {
    pub fn view(&self) -> Element<'_, Message> {
        row![button(icon_settings()).on_press(Message::SettingsOpen)].into()
    }
}
