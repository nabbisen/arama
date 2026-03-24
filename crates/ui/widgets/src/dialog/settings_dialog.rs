pub mod message;
mod update;
mod view;

mod tab;

use tab::{
    Tab, about::About, ai_settings::AiSettings, file_system_settings::FileSystemSettings,
    general_settings::GeneralSettings,
};

#[derive(Clone, Debug, Default)]
pub struct SettingsDialog {
    tab: Tab,
    general_settings: GeneralSettings,
    ai_settings: AiSettings,
    file_system_settings: FileSystemSettings,
    about: About,
}
