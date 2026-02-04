use std::path::PathBuf;

use iced::{Element, Task};

use crate::app::gallery::{self, Gallery};
use crate::app::util::load_images;

#[derive(Default)]
pub struct Window {
    gallery: Gallery,
}

pub enum Message {
    GalleryMessage(gallery::message::Message),
    ImagesLoaded(Vec<PathBuf>),
}

impl Window {
    pub fn new() -> (Self, Task<Message>) {
        (
            Self::default(),
            Task::perform(load_images("."), Message::ImagesLoaded), // カレントディレクトリをスキャン
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
            Message::ImagesLoaded(paths) => {
                self.gallery.image_paths = paths;
                Task::none()
            }
        }
    }
}
