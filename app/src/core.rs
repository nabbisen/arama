use std::path::PathBuf;

use app_json_settings::ConfigManager;
use arama_env::{
    IMAGE_EXTENSION_ALLOWLIST, Settings, VIDEO_EXTENSION_ALLOWLIST, cache_storage_path,
    cache_storage_path_v1, local_dir, target_media_type::TargetMediaType, validate_dir,
};
use arama_i18n::{Locale, set_locale};
use arama_ui_layout::{aside::Aside, footer::Footer, header::Header};
use arama_ui_main::views::{
    cache_page::CachePage,
    gallery::Gallery,
    setup::{self, Setup},
};
use arama_ui_widgets::{context_menu::ContextMenu, dialog};
use iced::{Point, Task};
use snora::{Toast, ToastIntent};

mod message;
mod settings;
mod subscription;
mod update;
mod view;

use message::Message;
use swdir::{DirNode, Swdir};

/// Top-level navigation pages rendered in the body slot.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum NavPage {
    Explorer,
    Cache,
    Settings,
}

pub struct App {
    setup: Setup,
    gallery: Gallery,
    header: Header,
    aside: Aside,
    footer: Footer,
    context_menu: ContextMenu,
    dialog: Option<Dialog>,
    toasts: Vec<Toast<Message>>,
    toast_id_counter: u64,
    settings: Settings,
    dir_node: Option<DirNode>,
    image_cell_path: Option<PathBuf>,
    processing: bool,
    /// Handle for the active thumbnail-cache or embedding task, used to
    /// abort it when the user switches to a different directory.
    task_handle: Option<iced::task::Handle>,
    /// Currently displayed top-level page.
    nav_page: NavPage,
    /// Settings page widget — persistent so tab state is preserved across
    /// navigation.
    settings_page: dialog::settings_dialog::SettingsDialog,
    /// Cache control page (RFC 004) — persistent so rows and filter
    /// survive navigation.
    cache_page: CachePage,
}

#[derive(Clone, Debug)]
enum Dialog {
    MediaFocusDialog(dialog::media_focus_dialog::MediaFocusDialog),
    SimilarPairsDialog(dialog::similar_pairs_dialog::SimilarPairsDialog),
}

impl App {
    pub fn start() -> iced::Result {
        iced::application(App::new, App::update, App::view)
            .subscription(App::subscription)
            .settings(App::settings())
            .run()
    }

    fn new() -> (Self, Task<Message>) {
        let processing = true;

        setup_validate();

        // One-time migration of the v1 cache database, if present.
        // A failure is not fatal — the cache is rebuilt lazily — but the
        // person should know recomputation is coming, so it surfaces as
        // a startup toast.
        let mut startup_toasts: Vec<Toast<Message>> = vec![];
        let mut toast_id_counter: u64 = 0;
        if let (Ok(v1), Ok(v2)) = (cache_storage_path_v1(), cache_storage_path()) {
            if let Err(err) = arama_cache::migrate_v1_if_present(&v1, &v2) {
                let id = toast_id_counter;
                toast_id_counter += 1;
                startup_toasts.push(Toast::new(
                    id,
                    ToastIntent::Error,
                    "Cache migration failed",
                    format!("The previous cache could not be imported and will be rebuilt: {err}"),
                    Message::ToastDismiss(id),
                ));
            }
        }

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
        let cache_lookup_strategy = settings.cache_lookup_strategy;
        let similarity_threshold = settings.similarity_threshold;
        let locale = settings.locale;
        set_locale(locale);

        let dir_node = dir_node(&root_dir_path, &target_media_type);

        let settings = Settings {
            root_dir_path,
            target_media_type,
            sub_dir_depth_limit,
            thumbnail_size,
            cache_lookup_strategy,
            similarity_threshold,
            locale,
        };

        let header = Header::new(&settings.root_dir_path);
        let aside = Aside::new(&settings.root_dir_path, false, false, processing);
        let dir_node_count = dir_node.count();
        let footer = Footer::new(thumbnail_size, dir_node_count.files, dir_node_count.dirs);
        let dialog = None;
        let settings_page = dialog::settings_dialog::SettingsDialog::new(
            &settings.target_media_type,
            settings.sub_dir_depth_limit,
            settings.similarity_threshold,
            settings.locale,
        );

        let gallery = Gallery::new().expect("failed to init gallery");

        let context_menu_point = Point::default();
        let context_menu = ContextMenu::new(context_menu_point, thumbnail_size);

        let task = if !setup.finished && !setup::util::ready() {
            Task::none()
        } else {
            Task::done(Message::CacheRequire(None))
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
                toasts: startup_toasts,
                toast_id_counter,
                settings,
                dir_node: Some(dir_node),
                image_cell_path: None,
                processing,
                task_handle: None,
                nav_page: NavPage::Explorer,
                settings_page,
                cache_page: CachePage::default(),
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
                cache_lookup_strategy: self.settings.cache_lookup_strategy,
                similarity_threshold: self.settings.similarity_threshold,
                locale: self.settings.locale,
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

    /// Push a transient error toast to the notification queue.
    fn push_error_toast(&mut self, title: impl Into<String>, body: impl Into<String>) {
        let id = self.toast_id_counter;
        self.toast_id_counter += 1;
        self.toasts.push(Toast::new(
            id,
            ToastIntent::Error,
            title,
            body,
            Message::ToastDismiss(id),
        ));
    }
}

fn setup_validate() {
    let local_dir = local_dir().expect("failed to get local dir");
    let _ = validate_dir(&local_dir);
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
