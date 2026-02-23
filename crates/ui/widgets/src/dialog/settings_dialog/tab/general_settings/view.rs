use iced::Element;
use iced::widget::text;

use super::{GeneralSettings, message::Message};

impl GeneralSettings {
    pub fn view(&self) -> Element<'_, Message> {
        // todo
        text("general settings here").into()
    }
}
