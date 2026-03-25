use std::path::PathBuf;

use app_json_settings::ConfigManager;
use arama_env::{
    IMAGE_EXTENSION_ALLOWLIST, Settings, VIDEO_EXTENSION_ALLOWLIST,
    target_media_type::TargetMediaType,
};
use arama_ui_layout::{aside::Aside, footer::Footer, header::Header};
use arama_ui_main::views::{
    gallery::Gallery,
    setup::{self, Setup},
};
use arama_ui_widgets::{context_menu::ContextMenu, dialog};
use iced::{Point, Task};

mod message;
mod subscription;
mod update;
mod view;

use message::Message;
use swdir::{DirNode, Swdir};

pub struct App {
    setup: Setup,
    gallery: Gallery,
    header: Header,
    aside: Aside,
    footer: Footer,
    context_menu: ContextMenu,
    dialog: Option<Dialog>,
    settings: Settings,
    dir_node: Option<DirNode>,
    image_cell_path: Option<PathBuf>,
    processing: bool,
}

#[derive(Clone, Debug)]
enum Dialog {
    MediaFocusDialog(dialog::media_focus_dialog::MediaFocusDialog),
    SimilarPairsDialog(dialog::similar_pairs_dialog::SimilarPairsDialog),
    SettingsDialog(dialog::settings_dialog::SettingsDialog),
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
            Ok(x) => x,
            Err(err) => {
                eprintln!("failed to load settings: {:?}", err);
                Settings::default()
            }
        };

        let root_dir_path = if settings.root_dir_path.is_empty() {
            "."
        } else {
            settings.root_dir_path.as_str()
        }
        .to_owned();
        let target_media_type = settings.target_media_type;
        let sub_dir_depth_limit = settings.sub_dir_depth_limit;
        let thumbnail_size = settings.thumbnail_size;

        let dir_node = dir_node(&root_dir_path, &target_media_type);

        let settings = Settings {
            root_dir_path,
            target_media_type,
            sub_dir_depth_limit,
            thumbnail_size,
        };

        let header = Header::default();
        let aside = Aside::new(&settings.root_dir_path, false, false, processing);
        let dir_node_count = dir_node.count();
        let footer = Footer::new(thumbnail_size, dir_node_count.files, dir_node_count.dirs);
        let dialog = None;

        let gallery = Gallery::new().expect("failed to init gallery");

        let context_menu_point = Point::default();
        let context_menu = ContextMenu::new(context_menu_point, thumbnail_size);

        let task = if !setup.finished && !setup::util::ready() {
            Task::none()
        } else {
            Task::done(Message::CacheRequire)
        };

        (
            Self {
                setup,
                gallery,
                header,
                aside,
                footer,
                context_menu,
                dialog,
                settings,
                dir_node: Some(dir_node),
                image_cell_path: None,
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
                thumbnail_size: self.settings.thumbnail_size,
            })
            .expect("failed to save config");
    }

    fn processing_on(&mut self) {
        self.processing = true;
        self.aside.set_processing(self.processing);
    }

    fn processing_off(&mut self) {
        self.processing = false;
        self.aside.set_processing(self.processing);
    }

    fn thumbnail_size_update(&mut self, thumbnail_size: u16) {
        self.settings.thumbnail_size = thumbnail_size;
        self.save_settings();
    }

    fn image_cell_path_update(&mut self, path: Option<PathBuf>) {
        self.image_cell_path = path;
        self.footer
            .update_image_cell_path(self.image_cell_path.to_owned());
    }
}

fn dir_node(root_dir_path: &str, target_media_type: &TargetMediaType) -> DirNode {
    let mut extension_allowlist: Vec<&str> = vec![];
    if target_media_type.include_image {
        extension_allowlist.extend(IMAGE_EXTENSION_ALLOWLIST);
    }
    if target_media_type.include_video {
        extension_allowlist.extend(VIDEO_EXTENSION_ALLOWLIST);
    }

    let dir_node = Swdir::default()
        .set_root_path(root_dir_path)
        .set_extension_allowlist(&extension_allowlist)
        // todo: error handling
        .expect("failed to get dir node")
        .walk();

    dir_node
}
