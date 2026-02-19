use app_json_settings::ConfigManager;
use arama_embedding::model::clip::has_model;
use arama_widget::dir_tree::{self, DirTree};
use iced::{Element, Subscription, Task, widget::row};

pub(super) mod components;
mod settings;
mod views;

use swdir::Swdir;
use views::gallery::{self, Gallery};

use crate::core::{
    components::common::model_loader::{self, ModelLoader},
    settings::Settings,
};

pub struct App {
    gallery: Gallery,
    dir_tree: DirTree,
    model_loader: ModelLoader,
}

pub enum Message {
    GalleryMessage(gallery::message::Message),
    DirTreeMessage(dir_tree::message::Message),
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

        let path = if let Some(settings) = settings.as_ref() {
            settings.root_dir_path.as_str()
        } else {
            "."
        };

        let dir_tree = DirTree::new(path, false, false);

        let model_loader = ModelLoader::default();

        // let task = if has_model() {
        //     gallery
        //         .default_task()
        //         .map(|message| Message::GalleryMessage(message))
        // } else {
        //     Task::none()
        // };

        (
            Self {
                gallery,
                dir_tree,
                model_loader,
            },
            // task,
            Task::none(),
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

        let dir_tree = self.dir_tree.view().map(Message::DirTreeMessage);

        row([dir_tree.into(), gallery.into()]).into()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::GalleryMessage(message) => self
                .gallery
                .update(message)
                .map(|message| Message::GalleryMessage(message)),
            Message::DirTreeMessage(message) => {
                let task = self
                    .dir_tree
                    .update(message.clone())
                    .map(|message| Message::DirTreeMessage(message));

                match message {
                    dir_tree::message::Message::DirClick(path) => {
                        // todo dir_node should be got from dir_tree
                        const EXTENSION_ALLOWLIST: &[&str; 6] =
                            &["png", "jpg", "jpeg", "webp", "gif", "bmp"];
                        let dir_node = Swdir::default()
                            .set_root_path(path)
                            .set_extension_allowlist(EXTENSION_ALLOWLIST)
                            .expect("failed to set allowlist")
                            .walk();
                        let _ = self
                            .gallery
                            .update(gallery::message::Message::DirSelect(dir_node));
                        return Task::none();
                    }
                    _ => (),
                }

                task
            }
            Message::ModelLoaderMessage(message) => {
                let task = self.model_loader.update(message);
                task.map(Message::ModelLoaderMessage)
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        // let subscriptions = [self.gallery.subscription().map(Message::GalleryMessage)];
        // Subscription::batch(subscriptions)
        Subscription::batch([])
    }
}
