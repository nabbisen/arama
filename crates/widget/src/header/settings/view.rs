use iced::Element;
use iced::widget::{button, row};

use super::Settings;
use super::message::Message;

impl Settings {
    pub fn view(&self) -> Element<'static, Message> {
        // todo
        row![button("⏱️"), button("✏️")].into()
    }
}
