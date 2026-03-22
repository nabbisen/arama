use std::path::PathBuf;

use app_json_settings::ConfigManager;
use arama_env::Settings;
use arama_ui_layout::{aside::Aside, footer::Footer, header::Header};
use arama_ui_main::views::{
    gallery::Gallery,
    setup::{self, Setup},
};
use arama_ui_widgets::dialog;
use iced::{Point, Task};

mod message;
mod subscription;
mod update;
mod view;

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
    settings: Settings,
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

        // todo: error handling
        let setup = Setup::default().expect("Failed to setup preparation");

        // todo: after setup
        let settings = match ConfigManager::<Settings>::new()
            .at_current_dir()
            .load_or_default()
        {
            Ok(x) => Some(x),
            Err(err) => {
                eprintln!("failed to load settings: {:?}", err);
                None
            }
        }
        .expect("failed to initialize settings");

        let root_dir_path = if settings.root_dir_path.is_empty() {
            "."
        } else {
            settings.root_dir_path.as_str()
        }
        .to_owned();
        let target_media_type = settings.target_media_type;
        let sub_dir_depth_limit = settings.sub_dir_depth_limit;
        let settings = Settings {
            root_dir_path,
            target_media_type,
            sub_dir_depth_limit,
        };

        let gallery = Gallery::new(
            &settings.root_dir_path,
            &settings.target_media_type,
            settings.sub_dir_depth_limit,
        )
        .expect("failed to init gallery");

        let header = Header::default();
        let aside = Aside::new(&settings.root_dir_path, false, false, processing);
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
                settings,
                processing,
            },
            task,
        )
    }

    fn save_settings(&self) {
        ConfigManager::new()
            .at_current_dir()
            .save(&Settings {
                root_dir_path: self.settings.root_dir_path.to_owned(),
                target_media_type: self.settings.target_media_type.to_owned(),
                sub_dir_depth_limit: self.settings.sub_dir_depth_limit,
            })
            .expect("failed to save config");
    }
}
