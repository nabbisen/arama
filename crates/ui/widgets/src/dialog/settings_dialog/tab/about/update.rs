use iced::Task;

use super::{About, message::Message};

impl About {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::RepositoryLinkClicked(url) => {
                let _ = webbrowser::open(&url);
            }
        }
        Task::none()
    }
}
