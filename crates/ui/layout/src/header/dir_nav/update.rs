use std::path::PathBuf;

use iced::Task;

use super::DirNav;
use super::message::{Event, Internal, Message};

impl DirNav {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Event(_) => return Task::none(),

            Message::Internal(message) => {
                match message {
                    Internal::Input(s) => self.input_str = s,
                    Internal::Submit => {
                        let path = PathBuf::from(&self.input_str);
                        if path.exists() {
                            self.original_path_str = self.input_str.clone();
                            return Task::done(Message::Event(Event::DirSelect(path)));
                        } else {
                            self.input_str = self.original_path_str.clone();
                        }
                    }
                    Internal::RfdOpen => {
                        return Task::perform(
                            async {
                                rfd::FileDialog::new()
                                    .set_title("Folder select")
                                    .pick_folder()
                            },
                            |path| Message::Internal(Internal::RfdClose(path)),
                        );
                    }
                    Internal::RfdClose(path) => {
                        if let Some(path) = path {
                            self.original_path_str = path.to_string_lossy().to_string();
                            self.input_str = self.original_path_str.clone();
                            return Task::done(Message::Event(Event::DirSelect(path)));
                        }
                    }
                }
                Task::none()
            }
        }
    }
}
