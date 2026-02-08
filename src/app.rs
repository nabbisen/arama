use std::path::Path;

use app_json_settings::ConfigManager;
use iced::{Element, Task};

pub(super) mod components;
mod settings;
mod utils;
mod views;

use views::gallery::{self, Gallery};

use crate::app::{
    components::common::model_loader::{self, ModelLoader},
    settings::Settings,
};

const SAFETENSORS_MODEL: &str = "model.safetensors";
const PYTORCH_MODEL: &str = "pytorch_model.bin";
const URL: &str = "https://huggingface.co/openai/clip-vit-base-patch32/resolve/main/pytorch_model.bin?download=true";

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
        iced::application(App::new, App::update, App::view).run()
    }

    fn new() -> (Self, Task<Message>) {
        let settings = ConfigManager::<Settings>::new().load_or_default();
        let gallery = if let Ok(settings) = settings {
            Gallery::new(&settings.root_dir_path)
        } else {
            Gallery::default()
        };

        let model_loader = ModelLoader::default();

        let task = gallery
            .default_task()
            .map(|message| Message::GalleryMessage(message));

        (
            Self {
                gallery,
                model_loader,
            },
            task,
        )
    }

    fn view(&self) -> Element<'_, Message> {
        if !Path::new(SAFETENSORS_MODEL).exists() {
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
}
