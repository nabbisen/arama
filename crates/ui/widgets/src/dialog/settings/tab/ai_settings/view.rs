use iced::Element;
use iced::widget::text;

use super::{AiSettings, message::Message};

impl AiSettings {
    pub fn view(&self) -> Element<'_, Message> {
        // todo
        text("ai settings here").into()
    }
}
