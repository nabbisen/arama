use iced::{Element, widget::row};

use super::{Header, message::Message};

impl Header {
    pub fn view(&self) -> Element<'static, Message> {
        row![
            self.dir_nav.view().map(Message::DirNavMessage),
            self.settings_nav.view().map(Message::SettingsNavMessage)
        ]
        .into()
    }
}
