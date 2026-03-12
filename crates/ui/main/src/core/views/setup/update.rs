use iced::Task;

use super::{Setup, message::Message};

impl Setup {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Download => {
                // let stream = Stream
                // Task::stream(stream)
                Task::none()
            }
            Message::Skip => Task::none(),
        }
    }
}
