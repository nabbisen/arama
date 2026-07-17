use std::path::{Path, PathBuf};

use app_json_settings::{ConfigError, ConfigManager};
use arama_env::{
    IMAGE_EXTENSION_ALLOWLIST, Settings, VIDEO_EXTENSION_ALLOWLIST, local_dir,
    target_media_type::TargetMediaType, validate_dir,
};
use arama_i18n::{set_locale, t};
use arama_sidecar::media::video::video_engine::{
    FfmpegToolchain, discovery::FfmpegDiscoveryRuntime,
};
use arama_ui_layout::{aside::Aside, footer::Footer, header::Header};
use arama_ui_main::views::{cache_page::CachePage, gallery::Gallery, setup::Setup};
use arama_ui_widgets::{context_menu::ContextMenu, dialog};
use iced::{Point, Task};
use snora::{Toast, ToastIntent};

mod message;
mod settings;
mod subscription;
mod update;
mod view;

use message::Message;
use swdir::{DirNode, FilterRule, Recurse, Swdir, WalkError};

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
    ffmpeg_runtime: FfmpegDiscoveryRuntime,
    ffmpeg_authority: update::ffmpeg::state::FfmpegAuthority<FfmpegToolchain>,
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
    /// Whether the Explorer aside tree pane is currently open.
    /// Closed by default so the gallery has full width on startup.
    /// Session-only: not persisted across restarts.
    aside_open: bool,
}

#[derive(Clone, Debug)]
enum Dialog {
    MediaFocusDialog(dialog::media_focus_dialog::MediaFocusDialog),
    SimilarPairsDialog(dialog::similar_pairs_dialog::SimilarPairsDialog),
}

fn setup_complete(finished: bool, ready: bool) -> bool {
    finished || ready
}

fn setup_became_complete(was_complete: bool, finished: bool, ready: bool) -> bool {
    !was_complete && setup_complete(finished, ready)
}

impl App {
    pub fn start() -> iced::Result {
        iced::application(App::new, App::update, App::view)
            .subscription(App::subscription)
            .settings(App::settings())
            .theme(app_theme)
            .run()
    }

    fn new() -> (Self, Task<Message>) {
        let mut startup_toasts: Vec<Toast<Message>> = vec![];
        let mut toast_id_counter: u64 = 0;
        let mut startup_notices: Vec<StartupNotice> = vec![];

        if let Some(notice) = setup_validation_notice() {
            startup_notices.push(notice);
        }

        let settings = match ConfigManager::<Settings>::new()
            .at_current_dir()
            .load_or_default()
        {
            Ok(x) => x,
            Err(err) => {
                startup_notices.push(StartupNotice::warning(
                    t("settings.load_error.title"),
                    format!(
                        "{}: {}",
                        t("settings.load_error.body"),
                        settings_error_message(&err)
                    ),
                ));
                Settings::default()
            }
        };
        let ffmpeg_runtime = FfmpegDiscoveryRuntime::default();
        let setup = match Setup::default() {
            Ok(s) => s,
            Err(err) => {
                startup_notices.push(StartupNotice::error(
                    "Setup initialization failed",
                    format!("The setup wizard could not be initialized: {err}"),
                ));
                Setup::fallback()
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
        let theme = settings.theme;
        let ffmpeg_location = settings.ffmpeg_location;
        arama_theme::set_theme(theme);

        let startup_root =
            startup_dir_node(&root_dir_path, &target_media_type, sub_dir_depth_limit);
        startup_notices.extend(startup_root.notices);
        let start_cache_on_startup =
            setup_complete(setup.finished, setup.ready()) && startup_root.dir_node.is_some();
        let processing = start_cache_on_startup;

        let settings = Settings {
            root_dir_path,
            target_media_type,
            sub_dir_depth_limit,
            thumbnail_size,
            cache_lookup_strategy,
            similarity_threshold,
            locale,
            theme,
            ffmpeg_location,
        };

        let header = Header::new(&settings.root_dir_path);
        let aside = Aside::new(processing);
        let dir_node_count = startup_root
            .dir_node
            .as_ref()
            .map(DirNode::count)
            .unwrap_or_default();
        let footer = Footer::new(thumbnail_size, dir_node_count.files, dir_node_count.dirs);
        let dialog = None;
        let settings_page = dialog::settings_dialog::SettingsDialog::new(
            &settings.target_media_type,
            settings.sub_dir_depth_limit,
            settings.similarity_threshold,
            settings.locale,
            settings.theme,
            settings.ffmpeg_location.clone(),
        );

        let gallery = Gallery::new();

        let context_menu_point = Point::default();
        let context_menu = ContextMenu::new(context_menu_point, thumbnail_size);

        push_startup_notices(&mut startup_toasts, &mut toast_id_counter, startup_notices);

        let setup_task = setup.initial_task().map(Message::SetupMessage);
        let mut ffmpeg_authority =
            update::ffmpeg::state::FfmpegAuthority::new(settings.ffmpeg_location.clone());
        let ffmpeg_epoch = ffmpeg_authority.begin(
            message::FfmpegRequestIntent::Startup,
            settings.ffmpeg_location.clone(),
        );
        let ffmpeg_task = update::ffmpeg::request_task(
            &ffmpeg_runtime,
            settings.ffmpeg_location.clone(),
            ffmpeg_epoch,
        );
        let cache_task = if start_cache_on_startup {
            Task::done(Message::CacheRequire(None))
        } else {
            Task::none()
        };
        let task = Task::batch([setup_task, ffmpeg_task, cache_task]);

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
                ffmpeg_runtime,
                ffmpeg_authority,
                dir_node: startup_root.dir_node,
                image_cell_path: None,
                processing,
                task_handle: None,
                nav_page: NavPage::Explorer,
                settings_page,
                cache_page: CachePage::default(),
                aside_open: false,
            },
            task,
        )
    }

    fn save_settings(&mut self) -> bool {
        match ConfigManager::new().at_current_dir().save(&Settings {
            root_dir_path: self.settings.root_dir_path.to_owned(),
            target_media_type: self.settings.target_media_type.to_owned(),
            sub_dir_depth_limit: self.settings.sub_dir_depth_limit,
            thumbnail_size: self.settings.thumbnail_size,
            cache_lookup_strategy: self.settings.cache_lookup_strategy,
            similarity_threshold: self.settings.similarity_threshold,
            locale: self.settings.locale,
            theme: self.settings.theme,
            ffmpeg_location: self.settings.ffmpeg_location.clone(),
        }) {
            Ok(()) => true,
            Err(err) => {
                self.push_error_toast(t("settings.save_error.title"), settings_error_message(&err));
                false
            }
        }
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
        self.push_toast(ToastIntent::Error, title, body);
    }

    fn push_success_toast(&mut self, title: impl Into<String>, body: impl Into<String>) {
        self.push_toast(ToastIntent::Success, title, body);
    }

    fn push_warning_toast(&mut self, title: impl Into<String>, body: impl Into<String>) {
        self.push_toast(ToastIntent::Warning, title, body);
    }

    fn push_toast(
        &mut self,
        intent: ToastIntent,
        title: impl Into<String>,
        body: impl Into<String>,
    ) {
        let id = self.toast_id_counter;
        self.toast_id_counter += 1;
        self.toasts.push(Toast::new(
            id,
            intent,
            title,
            body,
            Message::ToastDismiss(id),
        ));
    }
}

/// The iced base theme for the active preset (RFC 011, layer C).
///
/// A free function (rather than a closure) so the `Fn(&State) -> Theme`
/// bound resolves with a fully general state lifetime for iced's `ThemeFn`.
fn app_theme(_state: &App) -> iced::Theme {
    arama_theme::iced_theme()
}

struct StartupNotice {
    intent: ToastIntent,
    title: String,
    body: String,
}

impl StartupNotice {
    fn error(title: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            intent: ToastIntent::Error,
            title: title.into(),
            body: body.into(),
        }
    }

    fn warning(title: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            intent: ToastIntent::Warning,
            title: title.into(),
            body: body.into(),
        }
    }
}

struct StartupRoot {
    dir_node: Option<DirNode>,
    notices: Vec<StartupNotice>,
}

fn push_startup_notices(
    startup_toasts: &mut Vec<Toast<Message>>,
    toast_id_counter: &mut u64,
    notices: Vec<StartupNotice>,
) {
    for notice in notices {
        let id = *toast_id_counter;
        *toast_id_counter += 1;
        startup_toasts.push(Toast::new(
            id,
            notice.intent,
            notice.title,
            notice.body,
            Message::ToastDismiss(id),
        ));
    }
}

fn setup_validation_notice() -> Option<StartupNotice> {
    let local_dir = match local_dir() {
        Ok(path) => path,
        Err(err) => {
            return Some(StartupNotice::error(
                t("startup.local_setup_error.title"),
                format!("{}: {err}", t("startup.local_setup_error.body")),
            ));
        }
    };

    validate_dir(&local_dir).err().map(|err| {
        StartupNotice::warning(
            t("startup.local_setup_error.title"),
            format!(
                "{}: {} ({err})",
                t("startup.local_setup_error.body"),
                local_dir.display()
            ),
        )
    })
}

fn settings_error_message(err: &ConfigError) -> String {
    match err {
        ConfigError::Io(err) => format!("I/O error: {err}"),
        ConfigError::Serialize(err) => format!("JSON serialization error: {err}"),
        ConfigError::Deserialize(err) => format!("JSON deserialization error: {err}"),
        ConfigError::InvalidPathComponent(component) => {
            format!("Invalid settings path component: {component}")
        }
        ConfigError::Platform(error) => format!("Settings platform error: {error}"),
    }
}

fn startup_dir_node(
    root_dir_path: &str,
    target_media_type: &TargetMediaType,
    sub_dir_depth_limit: u8,
) -> StartupRoot {
    let root = Path::new(root_dir_path);
    if !root.is_dir() {
        return StartupRoot {
            dir_node: None,
            notices: vec![StartupNotice::warning(
                t("startup.root_dir_unavailable.title"),
                format!(
                    "{}: {}",
                    t("startup.root_dir_unavailable.body"),
                    root.display()
                ),
            )],
        };
    }

    let report = dir_node(root, target_media_type, sub_dir_depth_limit);
    let notices = if report.errors.is_empty() {
        vec![]
    } else {
        vec![StartupNotice::warning(
            t("startup.root_scan_warning.title"),
            format!(
                "{}: {}",
                t("startup.root_scan_warning.body"),
                walk_errors_summary(&report.errors)
            ),
        )]
    };

    StartupRoot {
        dir_node: Some(report.tree),
        notices,
    }
}

fn dir_node(
    root_dir_path: &Path,
    target_media_type: &TargetMediaType,
    sub_dir_depth_limit: u8,
) -> swdir::WalkReport {
    let mut extension_allowlist: Vec<&str> = vec![];
    if target_media_type.include_image {
        extension_allowlist.extend(IMAGE_EXTENSION_ALLOWLIST);
    }
    if target_media_type.include_video {
        extension_allowlist.extend(VIDEO_EXTENSION_ALLOWLIST);
    }

    // If the allowlist fails (e.g. an extension string is malformed), walk
    // without filtering rather than panicking.
    let recurse = if 0 < sub_dir_depth_limit {
        Recurse::Depth(sub_dir_depth_limit as usize)
    } else {
        Recurse::None
    };

    let scanner = Swdir::new().root_path(root_dir_path).recurse(recurse);
    let scanner = match FilterRule::extension_allowlist(extension_allowlist.iter().copied()) {
        Ok(filter) => scanner.filter(filter),
        Err(err) => {
            eprintln!("extension allowlist error (walking without filter): {err}");
            scanner
        }
    };

    scanner.walk()
}

fn walk_errors_summary(errors: &[WalkError]) -> String {
    let first = errors.first().map(ToString::to_string).unwrap_or_default();
    if errors.len() == 1 {
        first
    } else {
        format!("{first}; {} total scan errors", errors.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use app_json_settings::SaveMode;

    fn missing_test_path() -> PathBuf {
        std::env::temp_dir().join(format!("arama-missing-startup-root-{}", std::process::id()))
    }

    #[test]
    fn missing_startup_root_is_recoverable_without_cache_node() {
        let path = missing_test_path();
        assert!(!path.exists());

        let root = startup_dir_node(
            &path.to_string_lossy(),
            &TargetMediaType {
                include_image: true,
                include_video: true,
            },
            0,
        );

        assert!(root.dir_node.is_none());
        assert_eq!(root.notices.len(), 1);
    }

    #[test]
    fn walk_error_summary_omits_error_count_for_single_error() {
        let error = WalkError::Io {
            path: PathBuf::from("/not-readable"),
            source: std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied"),
        };
        let summary = walk_errors_summary(&[error]);

        assert!(summary.contains("not-readable"));
        assert!(!summary.contains("total scan errors"));
    }

    #[test]
    fn walk_error_summary_includes_error_count_for_multiple_errors() {
        let first = WalkError::Io {
            path: PathBuf::from("/not-readable"),
            source: std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied"),
        };
        let second = WalkError::Io {
            path: PathBuf::from("/also-not-readable"),
            source: std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied"),
        };
        let summary = walk_errors_summary(&[first, second]);

        assert!(summary.contains("not-readable"));
        assert!(summary.contains("2 total scan errors"));
    }

    #[test]
    fn failed_setup_requirement_does_not_trigger_cache_transition() {
        assert!(!setup_became_complete(false, false, false));
    }

    #[test]
    fn readiness_or_explicit_skip_triggers_cache_transition() {
        assert!(setup_became_complete(false, false, true));
        assert!(setup_became_complete(false, true, false));
        assert!(!setup_became_complete(true, true, true));
    }

    #[test]
    fn production_settings_manager_uses_atomic_replacement() {
        let manager = ConfigManager::<Settings>::new().at_current_dir();
        assert_eq!(manager.save_mode(), SaveMode::Atomic);
    }
}
