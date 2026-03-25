use iced::Task;

use crate::footer::thumbnail_size_slider;

use super::{Footer, message::Message};

impl Footer {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ThumbnailSizeSliderMessage(message) => {
                let task = self
                    .thumbnail_size_slider
                    .update(message.clone())
                    .map(Message::ThumbnailSizeSliderMessage);

                match message {
                    thumbnail_size_slider::message::Message::ValueChanged(value) => {
                        return Task::batch([
                            task,
                            Task::done(Message::ThumbnailSizeChanged(value)),
                        ]);
                    }
                }
            }
            Message::ThumbnailSizeChanged(_) => (),
        }
        Task::none()
    }
}
