use std::path::PathBuf;

use iced::Task;

use super::{
    CachePage,
    message::{Event, Internal, Message},
};

impl CachePage {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Event(_) => Task::none(),
            Message::Internal(message) => match message {
                Internal::FilterInput(s) => {
                    self.filter = s;
                    Task::none()
                }
                Internal::DirInput(s) => {
                    self.dir_input = s;
                    Task::none()
                }
                Internal::RefreshPressed => self.load_task(),
                Internal::CachePressed => {
                    let path = PathBuf::from(self.dir_input.trim());
                    // The app validates further; the page only emits the
                    // request for non-empty input.
                    if path.as_os_str().is_empty() {
                        Task::none()
                    } else {
                        Task::done(Message::Event(Event::CacheRequest(path)))
                    }
                }
                Internal::RowsLoaded(rows) => {
                    self.rows = rows;
                    self.busy = false;
                    self.loaded = true;
                    Task::none()
                }
            },
        }
    }
}
