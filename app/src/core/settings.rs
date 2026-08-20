use iced::Settings;
use lucide_icons::LUCIDE_FONT_BYTES;

use super::App;

impl App {
    /// RFC 043 §3.1: the default text size for every unannotated `text(`
    /// site comes from the active preset's `body` role rather than
    /// coinciding with iced's own default by chance. `Settings` is read
    /// once at application start, so a preset with different typography
    /// would not take effect until restart - not observable today, since
    /// all four built-in presets share `Typography::default_roles()`.
    pub fn settings() -> Settings {
        Settings {
            fonts: vec![LUCIDE_FONT_BYTES.into()],
            default_text_size: arama_theme::tokens().typography.body.size.into(),
            ..Default::default()
        }
    }
}
