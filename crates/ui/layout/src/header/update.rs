use iced::Task;

use crate::header::{dir_nav, settings_nav};

use super::{Header, message::Message};

impl Header {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::DirSelect(_) | Message::SimilarPairsDialogOpen | Message::SettingsOpen => (),
            Message::DirNavMessage(message) => {
                let _task = self.dir_nav.update(message.clone());
                match message {
                    dir_nav::message::Message::DirSelect(path) => {
                        return Task::done(Message::DirSelect(path));
                    }
                }
            }
            Message::SettingsNavMessage(message) => {
                let task = self
                    .settings_nav
                    .update(message)
                    .map(Message::SettingsNavMessage);

                match message {
                    settings_nav::message::Message::SettingsOpen => {
                        return Task::batch([task, Task::done(Message::SettingsOpen)]);
                    }
                }
            }
        }

        Task::none()
    }
}
