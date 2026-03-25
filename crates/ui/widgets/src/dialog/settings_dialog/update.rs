use iced::Task;

use crate::dialog::settings_dialog::tab::general_settings;

use super::{SettingsDialog, message::Message};

impl SettingsDialog {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::TargetMediaTypeChanged(_) => Task::none(),
            Message::TabSelect(tab) => {
                self.tab = tab;
                Task::none()
            }
            Message::GeneralSettingsTabMessage(message) => {
                let task = self
                    .general_settings
                    .update(message.clone())
                    .map(Message::GeneralSettingsTabMessage);

                match message {
                    general_settings::message::Message::TargetMediaTypeChanged(x) => {
                        Task::batch([task, Task::done(Message::TargetMediaTypeChanged(x))])
                    }
                }
            }
            Message::AiSettingsTabMessage(message) => {
                let task = self
                    .ai_settings
                    .update(message)
                    .map(Message::AiSettingsTabMessage);
                task
            }
            Message::FileSystemSettingsTabMessage(message) => {
                let task = self
                    .file_system_settings
                    .update(message)
                    .map(Message::FileSystemSettingsTabMessage);
                task
            }
            Message::AboutTabMessage(message) => {
                let task = self.about.update(message).map(Message::AboutTabMessage);
                task
            }
        }
    }
}
