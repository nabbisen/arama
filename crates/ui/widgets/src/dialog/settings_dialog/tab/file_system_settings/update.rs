use iced::Task;

use super::{FileSystemSettings, message::Message};

impl FileSystemSettings {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            // Task 039: this tab only requests deletion; the confirmation
            // dialog, the actual delete, and the outcome toast all live
            // at the app level, which `settings_dialog::update` bubbles
            // this into (see `settings_dialog::message::Message::CacheDeleteRequested`).
            Message::CacheDeleteRequested => {}
        }
        Task::none()
    }
}
