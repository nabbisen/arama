use iced::Settings;
use lucide_icons::LUCIDE_FONT_BYTES;

use super::App;

impl App {
    pub fn settings() -> Settings {
        Settings {
            fonts: vec![LUCIDE_FONT_BYTES.into()],
            ..Default::default()
        }
    }
}
