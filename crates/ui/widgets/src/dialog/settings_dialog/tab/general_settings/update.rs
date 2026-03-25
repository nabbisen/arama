use iced::Task;

use super::{GeneralSettings, message::Message};

impl GeneralSettings {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::TargetMediaTypeChanged(x) => {
                self.target_media_type = x;
            }
        }
        Task::none()
    }
}
