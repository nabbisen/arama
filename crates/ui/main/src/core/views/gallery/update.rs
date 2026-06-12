use iced::Task;

use super::{Gallery, message::Message};

impl Gallery {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ImageCellMessage(_) => (),
            Message::CursorExit => (),
            Message::FilterChanged(s) => self.filter = s,
            Message::FilterClear => self.filter.clear(),
        }
        Task::none()
    }

    /// Reset the filename filter. Called when the selected directory changes.
    pub fn clear_filter(&mut self) {
        self.filter.clear();
    }
}
