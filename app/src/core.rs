use app_json_settings::ConfigManager;
use arama_widget::dir_tree::DirTree;
use iced::Task;

pub(super) mod components;
mod message;
mod settings;
mod subscription;
mod update;
mod views;

use message::Message;
use views::gallery::{self, Gallery};

use crate::core::{components::common::model_loader::ModelLoader, settings::Settings};

pub struct App {
    gallery: Gallery,
    dir_tree: DirTree,
    model_loader: ModelLoader,
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
        let gallery = Gallery::new(settings.as_ref()).expect("failed to init gallery");

        let path = if let Some(settings) = settings.as_ref() {
            settings.root_dir_path.as_str()
        } else {
            "."
        };

        let dir_tree = DirTree::new(path, false, false);

        let model_loader = ModelLoader::default();

        let task = gallery
            .default_task()
            .map(|message| Message::GalleryMessage(message));

        (
            Self {
                gallery,
                dir_tree,
                model_loader,
            },
            task,
        )
    }
}
