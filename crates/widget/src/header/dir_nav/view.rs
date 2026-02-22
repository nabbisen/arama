use iced::Element;
use iced::widget::text_input;

use super::DirNav;
use super::message::Message;

impl DirNav {
    pub fn view(&self) -> Element<'_, Message> {
        text_input("test", "").into()
    }
}
