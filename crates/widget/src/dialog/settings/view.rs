use iced::Element;
use iced::widget::text;

use super::{Settings, message::Message};

impl Settings {
    pub fn view(&self) -> Element<'_, Message> {
        // todo
        text("settings dialog").into()
    }
}
