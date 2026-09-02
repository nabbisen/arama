use std::path::{Path, PathBuf};

use app_json_settings::{ConfigError, ConfigManager};
use arama_env::{
    IMAGE_EXTENSION_ALLOWLIST, Settings, VIDEO_EXTENSION_ALLOWLIST, diagnostic,
    target_media_type::TargetMediaType,
};
use arama_i18n::{set_locale, t, t_with};
use arama_sidecar::media::video::video_engine::{
    FfmpegToolchain, discovery::FfmpegDiscoveryRuntime,
};
use arama_ui_layout::{aside::Aside, footer::Footer, header::Header};
use arama_ui_main::views::{cache_page::CachePage, gallery::Gallery, setup::Setup};
use arama_ui_widgets::{context_menu::ContextMenu, dialog};
use iced::{Point, Task};
use snora::{Toast, ToastIntent};

mod data_locations;
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
    /// RFC 041, RFC 017 Fatal-startup tier: set when arama could not
    /// resolve or create even one of its three data locations (settings,
    /// data, cache) and therefore has nowhere to persist anything.
    /// `view()` renders only a blocking message when this is `Some` —
    /// every other field below still gets its ordinary (degraded)
    /// fallback value so construction never has to special-case around a
    /// half-built `App`, but none of it is shown.
    fatal_startup_error: Option<String>,
    /// `None` only alongside `fatal_startup_error: Some(_)` - there is
    /// nowhere to save to. `save_settings` no-ops in that case; `view()`
    /// never renders anything that could call it.
    settings_manager: Option<ConfigManager<Settings>>,
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
    /// RFC 044 §2.2: which skeleton zone F6 / Shift+F6 cycling currently
    /// treats as focused. arama's own state, not iced's - `next_zone` is
    /// pure and knows nothing about widgets. Drives 2.3's visible ring,
    /// gated by `focus_visible` below. Session-only, like `aside_open`: a
    /// page switch does not change which zones exist, so this
    /// deliberately does not reset on `NavTo`.
    focus_zone: snora::focus::FocusZone,
    /// RFC 044 §2.3: whether the ring should render at all. `focus_zone`
    /// defaults to `Body` before any key is pressed - rendering its ring
    /// unconditionally would draw a permanent border around the whole
    /// gallery for every mouse-only user from first launch, which is not
    /// what "focus must be visible when it *moves*" means. Set `true` the
    /// first time zone cycling actually runs; never reset afterward.
    focus_visible: bool,
}

#[derive(Clone, Debug)]
enum Dialog {
    MediaFocusDialog(dialog::media_focus_dialog::MediaFocusDialog),
    SimilarPairsDialog(dialog::similar_pairs_dialog::SimilarPairsDialog),
    /// Task 039: arama's first destructive-action confirmation surface.
    Confirm(dialog::confirm_dialog::ConfirmDialog),
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

        // RFC 041, RFC 017 Fatal-startup tier: resolving and creating the
        // three data locations is a startup precondition, checked before
        // anything else runs. A failure here is fatal, not a toast the
        // user might miss - arama has nowhere to persist anything. The
        // rest of this function still runs on its existing degraded
        // fallbacks (Settings::default(), Setup::fallback(), ...) so
        // construction never has to special-case a half-built App; view()
        // is what actually hides all of it behind the blocking message.
        let (locations, fatal_startup_error) = match data_locations::resolve_and_prepare_locations()
        {
            Ok(locations) => (Some(locations), None),
            Err(message) => (None, Some(message)),
        };

        // RFC 041 §7: resolved locations must be discoverable without a
        // debugger - this is the only output a headless/CI run produces,
        // so a failing migration or startup can still be diagnosed from
        // captured stderr alone.
        match &locations {
            Some(locations) => diagnostic(&data_locations::describe_locations(locations)),
            None => diagnostic(&data_locations::describe_unresolved(
                fatal_startup_error.as_deref().unwrap_or("unknown error"),
            )),
        }

        if let Some(locations) = &locations {
            startup_notices.extend(data_locations::migrate_application_data(locations));
        }

        let settings = match &locations {
            Some(locations) => match locations.settings_manager.load_or_default() {
                Ok(settings) => settings,
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
            },
            None => Settings::default(),
        };
        let ffmpeg_runtime = FfmpegDiscoveryRuntime::default();
        let setup = match Setup::default() {
            Ok(s) => s,
            Err(err) => {
                startup_notices.push(StartupNotice::error(
                    t("notice.setup_init_failed.title"),
                    format!("{}: {err}", t("notice.setup_init_failed.body")),
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
                fatal_startup_error,
                settings_manager: locations.map(|locations| locations.settings_manager),
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
                focus_zone: snora::focus::FocusZone::Body,
                focus_visible: false,
            },
            task,
        )
    }

    /// RFC 044 §2.2: which skeleton slots arama's layout populates, in
    /// `next_zone`'s vocabulary. `ZonePresence` describes slot
    /// *occupancy* (`AppLayout::header`/`side_bar`/`footer` being
    /// `Some`), not visual presence - arama composes its header into
    /// `body` (`view.rs`), so it never populates the `header` slot and
    /// `next_zone` will never stop there, by construction. Static for
    /// arama's layout today, so this takes no argument.
    pub(crate) fn zone_presence() -> snora::focus::ZonePresence {
        snora::focus::ZonePresence::none()
            .side_bar(true)
            .footer(true)
    }

    fn save_settings(&mut self) -> bool {
        // `None` only when startup hit the Fatal-startup path (RFC 041) -
        // nothing in that state is ever shown to the user, so a silent
        // no-op here is correct, not a swallowed error.
        let Some(manager) = &self.settings_manager else {
            return false;
        };
        match manager.save(&Settings {
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

pub(crate) struct StartupNotice {
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

    pub(crate) fn warning(title: impl Into<String>, body: impl Into<String>) -> Self {
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

fn settings_error_message(err: &ConfigError) -> String {
    match err {
        ConfigError::Io(err) => format!("{}: {err}", t("settings.error.io")),
        ConfigError::Serialize(err) => format!("{}: {err}", t("settings.error.serialize")),
        ConfigError::Deserialize(err) => format!("{}: {err}", t("settings.error.deserialize")),
        ConfigError::InvalidPathComponent(component) => {
            format!(
                "{}: {component}",
                t("settings.error.invalid_path_component")
            )
        }
        ConfigError::Platform(error) => format!("{}: {error}", t("settings.error.platform")),
        // ConfigError is not #[non_exhaustive]; upstream has already added a
        // variant in each of two prior minor releases without one. Keep the
        // specific arms above for better messages, but this arm is what
        // stops the next such minor from breaking this build. Unreachable
        // today, by construction — that's the insurance, not a mistake.
        #[allow(unreachable_patterns)]
        _ => format!("{}: {err}", t("settings.error.generic")),
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
            diagnostic(&format!(
                "extension allowlist error (walking without filter): {err}"
            ));
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
        format!(
            "{first}; {}",
            t_with(
                "startup.scan_errors_total",
                &[("{count}", &errors.len().to_string())]
            )
        )
    }
}

/// Task 023: `ARAMA_DATA_HOME` and the process's current directory are
/// both process-global state. Every test in this crate that mutates either
/// one (here in `core::tests` and in `core::view::tests`) holds this lock
/// for the full mutation window, so no two such tests can interleave
/// regardless of how the suite is invoked - not just under native-smoke's
/// one-`--exact`-test-per-process convention, but also a local
/// `cargo test -- --ignored` bulk run. The same defect class as Task 023's
/// Defect 1 (`env/src/dir.rs`'s `ARAMA_DATA_HOME` race); the pure-seam fix
/// used there doesn't apply cleanly to these tests because their entire
/// point is exercising the real, env-reading `App::new()` startup path
/// end-to-end, not a unit in front of it - so this is the task's own
/// second-preference route (serialise), used deliberately, not as a
/// smaller effort than the first.
#[cfg(test)]
pub(crate) static ARAMA_DATA_HOME_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[cfg(test)]
mod tests;
