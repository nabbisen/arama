use iced::{Element, Length::Fill, widget::container};

use super::{Footer, message::Message};

impl Footer {
    pub fn view(&self) -> Element<'static, Message> {
        container("test").width(Fill).into()
    }
}
