use std::path::PathBuf;

use app_json_settings::ConfigManager;
use arama_ui_layout::{aside::Aside, footer::Footer, header::Header};
use arama_ui_main::{
    components::gallery::gallery_settings::target_media_type::TargetMediaType,
    views::{
        gallery::Gallery,
        setup::{self, Setup},
    },
};
use arama_ui_widgets::dialog;
use iced::{Point, Task};

mod config;
mod message;
mod subscription;
mod update;
mod view;

use config::settings::Settings;
use message::Message;

pub struct App {
    setup: Setup,
    gallery: Gallery,
    header: Header,
    aside: Aside,
    footer: Footer,
    context_menu_point: Point,
    context_menu: ContextMenu,
    dialog: Option<Dialog>,
    target_media_type: TargetMediaType,
    processing: bool,
}

#[derive(Clone, Debug)]
enum Dialog {
    MediaFocusDialog(dialog::media_focus_dialog::MediaFocusDialog),
    SimilarPairsDialog(dialog::similar_pairs_dialog::SimilarPairsDialog),
    SettingsDialog(dialog::settings_dialog::SettingsDialog),
}

#[derive(Debug)]
enum ContextMenu {
    ImageCell(PathBuf),
    None,
}

impl App {
    pub fn start() -> iced::Result {
        iced::application(App::new, App::update, App::view)
            .subscription(App::subscription)
            .run()
    }

    fn new() -> (Self, Task<Message>) {
        let processing = true;
        let target_media_type = TargetMediaType::default();

        // todo: error handling
        let setup = Setup::default().expect("Failed to setup preparation");

        // todo: after setup
        let settings = match ConfigManager::<Settings>::new().load_or_default() {
            Ok(x) => Some(x),
            Err(err) => {
                eprintln!("failed to load settings: {:?}", err);
                None
            }
        };
        let root_dir_path = match settings.as_ref() {
            Some(x) => x.root_dir_path.as_str(),
            None => ".",
        };
        let gallery =
            Gallery::new(root_dir_path, &target_media_type).expect("failed to init gallery");

        let path = if let Some(settings) = settings.as_ref() {
            settings.root_dir_path.as_str()
        } else {
            "."
        };

        let header = Header::default();
        let aside = Aside::new(path, false, false, processing);
        let footer = Footer::default();
        let dialog = None;

        let task = if !setup.finished && !setup::util::ready() {
            Task::none()
        } else {
            gallery
                .default_task()
                .map(|message| Message::GalleryMessage(message))
        };

        (
            Self {
                setup,
                gallery,
                header,
                aside,
                footer,
                context_menu_point: Point::default(),
                context_menu: ContextMenu::None,
                dialog,
                target_media_type,
                processing,
            },
            task,
        )
    }
}
