use iced::Task;

use crate::header::{dir_nav, settings_nav};

use super::{Header, message::Message};

impl Header {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::DirSelect(_) => (),
            Message::SettingsClick => (),
            Message::DirNavMessage(message) => {
                let _task = self.dir_nav.update(message.clone());
                match message {
                    dir_nav::message::Message::DirSelect(path) => {
                        return Task::done(Message::DirSelect(path));
                    }
                }
            }
            Message::SettingsNavMessage(message) => {
                let output = self.settings_nav.update(message);
                match output {
                    settings_nav::output::Output::SettingsClick => {
                        return Task::done(Message::SettingsClick);
                    }
                }
            }
        }

        Task::none()
    }
}
