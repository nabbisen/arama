use std::path::PathBuf;

use arama_i18n::t;
use iced::Task;

use super::DirNav;
use super::message::{Event, Internal, Message};

impl DirNav {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Event(_) => Task::none(),

            Message::Internal(message) => {
                match message {
                    Internal::Input(s) => self.processing = s,
                    Internal::Submit => {
                        let path = PathBuf::from(&self.processing);
                        if path.exists() {
                            self.path = self.processing.to_owned();
                            return Task::done(Message::Event(Event::DirSelect(path)));
                        } else {
                            self.processing = self.path.to_owned();
                        }
                    }
                    Internal::RfdOpen => {
                        let title = t("header.dir_nav.folder_select_title");
                        return Task::perform(
                            async move { rfd::FileDialog::new().set_title(title).pick_folder() },
                            |path| Message::Internal(Internal::RfdClose(path)),
                        );
                    }
                    Internal::RfdClose(path) => {
                        if let Some(path) = path {
                            self.path = path.to_string_lossy().to_string();
                            self.processing = self.path.clone();
                            return Task::done(Message::Event(Event::DirSelect(path)));
                        }
                    }
                }
                Task::none()
            }
        }
    }
}
