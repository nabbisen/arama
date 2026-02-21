use iced::Element;
use iced::widget::{button, row};

use super::SettingsNav;
use super::message::Message;

impl SettingsNav {
    pub fn view(&self) -> Element<'static, Message> {
        // todo
        row![button("⏱️"), button("✏️")].into()
    }
}
