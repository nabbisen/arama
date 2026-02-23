use iced::Task;

use super::{Settings, message::Message};

impl Settings {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::TabSelect(tab) => self.tab = tab,
            Message::GeneralSettingsTabMessage(message) => {
                let task = self.general_settings.update(message);
                return task.map(Message::GeneralSettingsTabMessage);
            }
            Message::AiSettingsTabMessage(message) => {
                let task = self.ai_settings.update(message);
                return task.map(Message::AiSettingsTabMessage);
            }
            Message::FileSystemSettingsTabMessage(message) => {
                let task = self.file_system_settings.update(message);
                return task.map(Message::FileSystemSettingsTabMessage);
            }
        }
        Task::none()
    }
}
