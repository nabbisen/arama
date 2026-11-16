use iced::Settings;
use lucide_icons::LUCIDE_FONT_BYTES;

use super::App;

impl App {
    /// RFC 043 §3.1: the default text size for every unannotated `text(`
    /// site comes from the active preset's `body` role rather than
    /// coinciding with iced's own default by chance. `Settings` is read
    /// once at application start, so a changed preset or text-size step
    /// (RFC 052 §2.2) does not take effect until restart. This function
    /// runs before `App::new()` ever does (`App::start`'s own
    /// `.settings(App::settings())` is evaluated before `.run()` starts
    /// the event loop), so `arama_theme::tokens()` only reflects the
    /// user's persisted choice because `App::start` preloads
    /// `set_theme`/`set_text_scale` first - see `preload_theme_and_
    /// text_scale` in `core.rs`.
    pub fn settings() -> Settings {
        Settings {
            fonts: vec![LUCIDE_FONT_BYTES.into()],
            default_text_size: arama_theme::tokens().typography.body.size.into(),
            ..Default::default()
        }
    }
}
