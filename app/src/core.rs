use std::path::{Path, PathBuf};

use app_json_settings::{ConfigError, ConfigManager};
use arama_env::{
    IMAGE_EXTENSION_ALLOWLIST, Settings, VIDEO_EXTENSION_ALLOWLIST,
    target_media_type::TargetMediaType,
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
            Some(locations) => eprintln!("{}", data_locations::describe_locations(locations)),
            None => eprintln!(
                "{}",
                data_locations::describe_unresolved(
                    fatal_startup_error.as_deref().unwrap_or("unknown error")
                )
            ),
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
        ConfigError::Io(err) => format!("I/O error: {err}"),
        ConfigError::Serialize(err) => format!("JSON serialization error: {err}"),
        ConfigError::Deserialize(err) => format!("JSON deserialization error: {err}"),
        ConfigError::InvalidPathComponent(component) => {
            format!("Invalid settings path component: {component}")
        }
        ConfigError::Platform(error) => format!("Settings platform error: {error}"),
        // ConfigError is not #[non_exhaustive]; upstream has already added a
        // variant in each of two prior minor releases without one. Keep the
        // specific arms above for better messages, but this arm is what
        // stops the next such minor from breaking this build. Unreachable
        // today, by construction — that's the insurance, not a mistake.
        #[allow(unreachable_patterns)]
        _ => format!("Settings error: {err}"),
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
        let manager = ConfigManager::<Settings>::new();
        assert_eq!(manager.save_mode(), SaveMode::Atomic);
    }

    // --- RFC 041 §7 verification --------------------------------------
    //
    // `nothing_is_written_outside_arama_data_home` runs everywhere: it only
    // ever touches an `ARAMA_DATA_HOME` scratch directory, so it is exactly
    // as safe as `core::view::tests`'s existing `App::new()` test.
    //
    // The other three are gated `#[ignore]` and meant for
    // `native-smoke.yaml` only (`cargo test -p arama --lib --locked --
    // --ignored --exact <name> --nocapture`, matching that workflow's own
    // convention for every other environment-touching check). Even with
    // `ARAMA_DATA_HOME` covering the *new* side of a migration, these still
    // create `.arama-local`/`.arama-cache` next to the test binary itself
    // (`legacy_local_dir`/`legacy_cache_dir` are unconditionally
    // exe-relative, on purpose - that's the pre-041 behaviour under test)
    // and briefly change the process's current directory. Both are
    // self-cleaning on success, but only worth doing on an ephemeral CI
    // runner, not a developer's own machine.

    fn scratch_data_home(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!("arama-native-smoke-{}-{label}", std::process::id()))
    }

    #[test]
    fn nothing_is_written_outside_arama_data_home() {
        let _guard = ARAMA_DATA_HOME_TEST_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let previous = std::env::var_os(arama_env::DATA_HOME_ENV_VAR);
        let scratch = scratch_data_home("nothing-outside-data-home");
        let exe_parent = std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf();
        let before: std::collections::BTreeSet<_> = std::fs::read_dir(&exe_parent)
            .unwrap()
            .filter_map(|entry| entry.ok().map(|entry| entry.file_name()))
            .collect();

        unsafe {
            std::env::set_var(arama_env::DATA_HOME_ENV_VAR, &scratch);
        }
        let mut app = App::new().0;
        app.settings.root_dir_path = "native-smoke-marker".to_owned();
        app.save_settings();
        unsafe {
            match &previous {
                Some(value) => std::env::set_var(arama_env::DATA_HOME_ENV_VAR, value),
                None => std::env::remove_var(arama_env::DATA_HOME_ENV_VAR),
            }
        }
        let _ = std::fs::remove_dir_all(&scratch);

        let after: std::collections::BTreeSet<_> = std::fs::read_dir(&exe_parent)
            .unwrap()
            .filter_map(|entry| entry.ok().map(|entry| entry.file_name()))
            .collect();
        assert_eq!(
            before, after,
            "App::new()+save_settings() must not write anything next to the executable"
        );
    }

    #[test]
    #[ignore]
    fn native_smoke_settings_path_is_independent_of_working_directory() {
        // No ARAMA_DATA_HOME override: this deliberately exercises the real
        // `ConfigManager::for_app` platform resolution, the thing §4.1
        // fixed. The only real-machine effect is `mkdir -p` of the real
        // settings *directory* (never settings.json itself, and never an
        // overwrite) - resolving a location does not write the file.
        let _guard = ARAMA_DATA_HOME_TEST_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let original_cwd = std::env::current_dir().unwrap();
        let dir_a = scratch_data_home("cwd-independence-a");
        let dir_b = scratch_data_home("cwd-independence-b");
        std::fs::create_dir_all(&dir_a).unwrap();
        std::fs::create_dir_all(&dir_b).unwrap();

        std::env::set_current_dir(&dir_a).unwrap();
        let path_a = data_locations::resolve_and_prepare_locations()
            .unwrap()
            .settings_manager
            .path();
        std::env::set_current_dir(&dir_b).unwrap();
        let path_b = data_locations::resolve_and_prepare_locations()
            .unwrap()
            .settings_manager
            .path();
        std::env::set_current_dir(&original_cwd).unwrap();
        let _ = std::fs::remove_dir_all(&dir_a);
        let _ = std::fs::remove_dir_all(&dir_b);

        assert_eq!(
            path_a, path_b,
            "settings must resolve to the same path regardless of the working directory \
             arama was launched from (RFC 041 §4.1's defect)"
        );
    }

    #[test]
    #[ignore]
    fn native_smoke_migration_moves_settings_models_and_cache_from_the_legacy_layout() {
        let _guard = ARAMA_DATA_HOME_TEST_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let original_cwd = std::env::current_dir().unwrap();
        let legacy_settings_cwd = scratch_data_home("migration-legacy-settings-cwd");
        std::fs::create_dir_all(&legacy_settings_cwd).unwrap();
        let data_home = scratch_data_home("migration-new-side");

        let legacy_local = arama_env::legacy_local_dir().unwrap();
        let legacy_cache = arama_env::legacy_cache_dir().unwrap();
        let _ = std::fs::remove_dir_all(&legacy_local);
        let _ = std::fs::remove_dir_all(&legacy_cache);
        std::fs::create_dir_all(&legacy_local).unwrap();
        std::fs::create_dir_all(&legacy_cache).unwrap();
        std::fs::write(legacy_local.join("model.marker"), b"legacy-model").unwrap();
        std::fs::write(legacy_cache.join("cache.marker"), b"legacy-cache").unwrap();

        std::env::set_current_dir(&legacy_settings_cwd).unwrap();
        ConfigManager::<Settings>::new()
            .at_current_dir()
            .save(&Settings {
                root_dir_path: "native-smoke-legacy-marker".to_owned(),
                ..Settings::default()
            })
            .unwrap();
        unsafe {
            std::env::set_var(arama_env::DATA_HOME_ENV_VAR, &data_home);
        }

        let locations = data_locations::resolve_and_prepare_locations().unwrap();
        let notices = data_locations::migrate_application_data(&locations);

        unsafe {
            std::env::remove_var(arama_env::DATA_HOME_ENV_VAR);
        }
        std::env::set_current_dir(&original_cwd).unwrap();
        let _ = std::fs::remove_dir_all(&legacy_settings_cwd);

        assert!(
            notices.is_empty(),
            "migration should succeed without warnings: {:?}",
            notices.iter().map(|n| &n.title).collect::<Vec<_>>()
        );
        assert_eq!(
            locations.settings_manager.load().unwrap().root_dir_path,
            "native-smoke-legacy-marker"
        );
        assert_eq!(
            std::fs::read(locations.local_dir.join("model.marker")).unwrap(),
            b"legacy-model"
        );
        assert_eq!(
            std::fs::read(locations.cache_dir.join("cache.marker")).unwrap(),
            b"legacy-cache"
        );
        assert!(
            !legacy_local.exists(),
            "the legacy data directory must be moved, not left behind next to the executable"
        );
        assert!(
            !legacy_cache.exists(),
            "the legacy cache directory must be moved, not left behind next to the executable"
        );

        let _ = std::fs::remove_dir_all(&data_home);
    }

    #[test]
    #[ignore]
    fn native_smoke_migration_prefers_new_location_when_both_are_populated() {
        let _guard = ARAMA_DATA_HOME_TEST_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let data_home = scratch_data_home("migration-both-populated");
        let legacy_local = arama_env::legacy_local_dir().unwrap();
        let _ = std::fs::remove_dir_all(&legacy_local);
        std::fs::create_dir_all(&legacy_local).unwrap();
        std::fs::write(legacy_local.join("old.marker"), b"old").unwrap();

        unsafe {
            std::env::set_var(arama_env::DATA_HOME_ENV_VAR, &data_home);
        }
        let locations = data_locations::resolve_and_prepare_locations().unwrap();
        std::fs::write(locations.local_dir.join("current.marker"), b"current").unwrap();

        let notices = data_locations::migrate_application_data(&locations);

        unsafe {
            std::env::remove_var(arama_env::DATA_HOME_ENV_VAR);
        }

        assert!(notices.is_empty());
        assert!(locations.local_dir.join("current.marker").exists());
        assert!(!locations.local_dir.join("old.marker").exists());
        assert!(
            legacy_local.join("old.marker").exists(),
            "the new location wins; the legacy directory must be left untouched, not deleted"
        );

        let _ = std::fs::remove_dir_all(&data_home);
        let _ = std::fs::remove_dir_all(&legacy_local);
    }

    // --- RFC 044 Phase 0.1: what does the keyboard do in arama today? --
    //
    // Answered by running, not reading (handoff §4): `Simulator::tap_key`
    // returns `event::Status::{Captured, Ignored}`, so "does anything
    // consume this key today" is assertable in-process, per key, without
    // a window or a compositor. arama installs no keyboard subscription
    // and no focus operation anywhere (`subscription.rs` carries only the
    // toast sweep), so the expectation is `Ignored` everywhere except
    // wherever iced's own widgets already claim a key on arama's behalf -
    // this test exists to find out where that is, not to assert a
    // pre-decided answer.
    #[test]
    fn phase_0_1_keyboard_baseline_on_gallery_and_settings() {
        use iced::keyboard::{Key, key::Named};

        let _guard = ARAMA_DATA_HOME_TEST_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let previous = std::env::var_os(arama_env::DATA_HOME_ENV_VAR);
        let scratch = scratch_data_home("phase-0-1-keyboard-baseline");
        unsafe {
            std::env::set_var(arama_env::DATA_HOME_ENV_VAR, &scratch);
        }

        let mut app = App::new().0;
        // Setup is not finished in a scratch profile (no CLIP model), so
        // `view()` would render the setup wizard rather than the gallery.
        // Phase 0.1 asks about the gallery and settings screens
        // specifically (handoff §4) - force setup complete the same way
        // `Message::Skip` would, without depending on that message's
        // other side effects.
        app.setup.finished = true;

        let keys = [
            ("Tab", Key::Named(Named::Tab)),
            ("Escape", Key::Named(Named::Escape)),
            ("Enter", Key::Named(Named::Enter)),
            ("ArrowDown", Key::Named(Named::ArrowDown)),
            ("F6", Key::Named(Named::F6)),
        ];

        eprintln!("=== Phase 0.1 keyboard baseline (RFC 044) ===");
        for (screen, nav) in [("gallery", None), ("settings", Some(NavPage::Settings))] {
            if let Some(nav) = nav {
                let _ = app.update(Message::NavTo(nav));
            }
            let element = app.view();
            let mut simulator = iced_test::Simulator::new(element);
            for (name, key) in &keys {
                let status = simulator.tap_key(key.clone());
                eprintln!("{screen:9} {name:10} -> {status:?}");
            }
        }

        unsafe {
            match &previous {
                Some(value) => std::env::set_var(arama_env::DATA_HOME_ENV_VAR, value),
                None => std::env::remove_var(arama_env::DATA_HOME_ENV_VAR),
            }
        }
        let _ = std::fs::remove_dir_all(&scratch);
    }

    // --- RFC 044 Tier 2: in-process, headless, on real App state -------
    //
    // Phase 0.1 (above) already proved F6 is `Ignored` by every widget on
    // the gallery and settings screens - so in the real application it
    // reaches `subscription.rs`'s keyboard listener and becomes
    // `Message::KeyPressed`. This test drives that exact message through
    // `App::update` (not `Simulator`, which only exercises the widget
    // tree, not `update`) and asserts the zone this RFC's own state
    // actually moved to - no window, no compositor, no rendering.
    #[test]
    fn tier_2_f6_moves_focus_zone_through_real_app_update() {
        use iced::keyboard::{Key, Modifiers, key::Named};

        let _guard = ARAMA_DATA_HOME_TEST_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let previous = std::env::var_os(arama_env::DATA_HOME_ENV_VAR);
        let scratch = scratch_data_home("tier-2-f6-focus-zone");
        unsafe {
            std::env::set_var(arama_env::DATA_HOME_ENV_VAR, &scratch);
        }

        let mut app = App::new().0;
        assert_eq!(
            app.focus_zone,
            snora::focus::FocusZone::Body,
            "starting zone, before any cycling"
        );
        assert!(
            !app.focus_visible,
            "the ring must not render before a keyboard user is known to exist"
        );

        let f6 = Key::Named(Named::F6);
        let _ = app.update(Message::KeyPressed(f6.clone(), Modifiers::default()));
        assert_eq!(
            app.focus_zone,
            snora::focus::FocusZone::Footer,
            "forward from Body skips the never-present Header and lands on Footer"
        );
        assert!(
            app.focus_visible,
            "the first real cycle must turn the ring on"
        );

        let _ = app.update(Message::KeyPressed(f6.clone(), Modifiers::default()));
        assert_eq!(
            app.focus_zone,
            snora::focus::FocusZone::SideBar,
            "forward from Footer wraps past Header to SideBar"
        );

        let _ = app.update(Message::KeyPressed(f6, Modifiers::SHIFT));
        assert_eq!(
            app.focus_zone,
            snora::focus::FocusZone::Footer,
            "Shift+F6 from SideBar goes backward, past Header, to Footer"
        );

        unsafe {
            match &previous {
                Some(value) => std::env::set_var(arama_env::DATA_HOME_ENV_VAR, value),
                None => std::env::remove_var(arama_env::DATA_HOME_ENV_VAR),
            }
        }
        let _ = std::fs::remove_dir_all(&scratch);
    }

    #[test]
    fn tier_2_escape_closes_a_real_open_dialog() {
        let _guard = ARAMA_DATA_HOME_TEST_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let previous = std::env::var_os(arama_env::DATA_HOME_ENV_VAR);
        let scratch = scratch_data_home("tier-2-escape-closes-dialog");
        unsafe {
            std::env::set_var(arama_env::DATA_HOME_ENV_VAR, &scratch);
        }

        let mut app = App::new().0;
        let _ = app.update(Message::KeyPressed(
            iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape),
            iced::keyboard::Modifiers::default(),
        ));
        // Nothing open yet - Escape must not panic or synthesize a close.
        assert!(app.dialog.is_none());

        app.dialog = Some(Dialog::MediaFocusDialog(
            dialog::media_focus_dialog::MediaFocusDialog::new(
                PathBuf::from("/does/not/need/to/exist.jpg"),
                arama_env::cache_lookup_strategy::CacheLookupStrategy::CurrentDirOnly,
                0.86,
                None,
            ),
        ));
        assert!(app.dialog.is_some());

        let _ = app.update(Message::KeyPressed(
            iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape),
            iced::keyboard::Modifiers::default(),
        ));
        assert!(
            app.dialog.is_none(),
            "Escape must close the real dialog through the real update path"
        );

        unsafe {
            match &previous {
                Some(value) => std::env::set_var(arama_env::DATA_HOME_ENV_VAR, value),
                None => std::env::remove_var(arama_env::DATA_HOME_ENV_VAR),
            }
        }
        let _ = std::fs::remove_dir_all(&scratch);
    }

    // --- RFC 044 §0.2b: does `Simulator::snapshot` work here at all? ---
    //
    // snora has never run this path (RFC-011-D chose semantic over pixel
    // testing) and asked to hear whether it works for a focus indicator.
    // This is that experiment, not a permanent regression suite - Tier
    // 3's own footgun (`matches_image`/`matches_hash` auto-create *and*
    // auto-pass on a missing reference) means a real regression suite
    // needs checked-in reference files and a documented regeneration
    // process, which is a separate decision from "does the mechanism
    // work." Reported in the review package either way.
    #[test]
    fn phase_0_2b_does_simulator_snapshot_render_here() {
        use iced::keyboard::{Key, Modifiers, key::Named};

        let _guard = ARAMA_DATA_HOME_TEST_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let previous = std::env::var_os(arama_env::DATA_HOME_ENV_VAR);
        let scratch = scratch_data_home("phase-0-2b-snapshot-experiment");
        unsafe {
            std::env::set_var(arama_env::DATA_HOME_ENV_VAR, &scratch);
        }

        let mut app = App::new().0;
        app.setup.finished = true;
        // Move focus so the ring is actually present in the frame this
        // snapshots - a snapshot of the pre-`focus_visible` state would
        // prove nothing about the indicator.
        let _ = app.update(Message::KeyPressed(
            Key::Named(Named::F6),
            Modifiers::default(),
        ));
        assert!(app.focus_visible);

        let element = app.view();
        let mut simulator = iced_test::Simulator::new(element);
        let theme = arama_theme::iced_theme();
        let result = simulator.snapshot(&theme);

        eprintln!(
            "=== Phase 0.2b: Simulator::snapshot result: {:?} ===",
            result.as_ref().map(|_| "Ok")
        );
        let snapshot = result.expect(
            "Simulator::snapshot must at least render successfully on this hardware \
             for Tier 3 to be a real option",
        );

        // Exercise the actual comparison codepath too, not just draw +
        // screenshot - on a scratch path so this run's baseline is
        // thrown away rather than becoming a permanent reference nobody
        // reviewed.
        let hash_path = scratch.join("phase-0-2b-snapshot");
        let first_call = snapshot
            .matches_hash(&hash_path)
            .expect("hashing the rendered frame must not fail");
        assert!(first_call, "a freshly created reference must match itself");
        let second_call = snapshot
            .matches_hash(&hash_path)
            .expect("hashing the rendered frame must not fail");
        assert!(
            second_call,
            "the same App state must render identically across two snapshot calls"
        );
        eprintln!("=== Phase 0.2b: matches_hash round-trip succeeded ===");

        unsafe {
            match &previous {
                Some(value) => std::env::set_var(arama_env::DATA_HOME_ENV_VAR, value),
                None => std::env::remove_var(arama_env::DATA_HOME_ENV_VAR),
            }
        }
        let _ = std::fs::remove_dir_all(&scratch);
    }
}
