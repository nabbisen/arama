use super::SettingsNav;
use super::{message::Message, output::Output};

impl SettingsNav {
    pub fn update(&mut self, message: Message) -> Output {
        match message {
            Message::SettingsClick => Output::SettingsClick,
        }
    }
}
