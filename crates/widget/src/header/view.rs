use iced::{Element, widget::row};

use super::{Header, message::Message};

impl Header {
    pub fn view(&self) -> Element<'static, Message> {
        row![
            self.dir_nav.view().map(Message::DirNavMessage),
            self.settings.view().map(Message::SettingsMessage)
        ]
        .into()
    }
}
