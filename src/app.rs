use iced::{Element, Task};

pub(super) mod components;
mod utils;
mod views;

use views::gallery::{self, Gallery};

pub struct App {
    gallery: Gallery,
}

pub enum Message {
    GalleryMessage(gallery::message::Message),
}

impl App {
    pub fn start() -> iced::Result {
        iced::application(App::new, App::update, App::view).run()
    }

    fn new() -> (Self, Task<Message>) {
        let gallery = Gallery::default();
        let task = gallery
            .default_task()
            .map(|message| Message::GalleryMessage(message));
        (Self { gallery }, task)
    }

    fn view(&self) -> Element<'_, Message> {
        let gallery = self
            .gallery
            .view()
            .map(|message| Message::GalleryMessage(message));
        gallery.into()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::GalleryMessage(message) => self
                .gallery
                .update(message)
                .map(|message| Message::GalleryMessage(message)),
        }
    }
}
