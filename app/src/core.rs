use app_json_settings::ConfigManager;
use arama_widget::{aside::Aside, dialog, footer::Footer, header::Header};
use iced::Task;

pub(super) mod components;
mod message;
mod settings;
mod subscription;
mod update;
mod views;

use message::Message;
use views::gallery::{self, Gallery};

use crate::core::settings::Settings;

pub struct App {
    gallery: Gallery,
    header: Header,
    aside: Aside,
    footer: Footer,
    dialog: Option<Dialog>,
}

enum Dialog {
    Settings(dialog::settings::Settings),
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

        let header = Header::default();
        let aside = Aside::new(path, false, false);
        let footer = Footer::default();
        let dialog = None;

        let task = gallery
            .default_task()
            .map(|message| Message::GalleryMessage(message));

        (
            Self {
                gallery,
                header,
                aside,
                footer,
                dialog,
            },
            task,
        )
    }
}
