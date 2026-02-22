use crate::header::{dir_nav, settings_nav};

use super::{Header, message::Message, output::Output};

impl Header {
    pub fn update(&mut self, message: Message) -> Output {
        match message {
            Message::DirNavMessage(message) => {
                let output = self.dir_nav.update(message);
                match output {
                    dir_nav::output::Output::DirSelect(path) => Output::DirSelect(path),
                }
            }
            Message::SettingsNavMessage(message) => {
                let output = self.settings_nav.update(message);
                match output {
                    settings_nav::output::Output::SettingsClick => Output::SettingsClick,
                }
            }
        }
    }
}
