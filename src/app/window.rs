use iced::{Element, Task};

use crate::app::gallery::{self, Gallery};

pub struct Window {
    gallery: Gallery,
}

pub enum Message {
    GalleryMessage(gallery::message::Message),
}

impl Window {
    pub fn new() -> (Self, Task<Message>) {
        (
            Self {
                gallery: Gallery::default(),
            },
            Gallery::default_task().map(|message| Message::GalleryMessage(message)),
        )
    }

    pub fn view(&self) -> Element<'_, Message> {
        let gallery = self
            .gallery
            .view()
            .map(|message| Message::GalleryMessage(message));
        gallery.into()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::GalleryMessage(message) => self
                .gallery
                .update(message)
                .map(|message| Message::GalleryMessage(message)),
        }
    }
}
