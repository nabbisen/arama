use app_json_settings::ConfigManager;
use arama_embedding::model::clip::has_model;
use iced::{Element, Subscription, Task};

pub(super) mod components;
mod settings;
mod views;

use views::gallery::{self, Gallery};

use crate::core::{
    components::common::model_loader::{self, ModelLoader},
    settings::Settings,
};

pub struct App {
    gallery: Gallery,
    model_loader: ModelLoader,
}

pub enum Message {
    GalleryMessage(gallery::message::Message),
    ModelLoaderMessage(model_loader::Message),
}

impl App {
    pub fn start() -> iced::Result {
        iced::application(App::new, App::update, App::view)
            .subscription(App::subscription)
            .run()
    }

    fn new() -> (Self, Task<Message>) {
        let settings = match ConfigManager::<Settings>::new().load_or_default() {
            Ok(x) => Some(x),
            Err(err) => {
                eprintln!("failed to load settings: {:?}", err);
                None
            }
        };
        let gallery = Gallery::new(settings.as_ref());

        let model_loader = ModelLoader::default();

        let task = if has_model() {
            gallery
                .default_task()
                .map(|message| Message::GalleryMessage(message))
        } else {
            Task::none()
        };

        (
            Self {
                gallery,
                model_loader,
            },
            task,
        )
    }

    fn view(&self) -> Element<'_, Message> {
        if !has_model() {
            return self
                .model_loader
                .view()
                .map(Message::ModelLoaderMessage)
                .into();
        }

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
            Message::ModelLoaderMessage(message) => {
                let task = self.model_loader.update(message);
                task.map(Message::ModelLoaderMessage)
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        let subscriptions = [self.gallery.subscription().map(Message::GalleryMessage)];
        Subscription::batch(subscriptions)
    }
}
