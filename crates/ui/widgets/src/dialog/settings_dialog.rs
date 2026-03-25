pub mod message;
mod update;
mod view;

mod tab;

use arama_env::target_media_type::TargetMediaType;
use tab::{
    Tab, about::About, ai_settings::AiSettings, file_system_settings::FileSystemSettings,
    general_settings::GeneralSettings,
};

#[derive(Clone, Debug)]
pub struct SettingsDialog {
    tab: Tab,
    general_settings: GeneralSettings,
    ai_settings: AiSettings,
    file_system_settings: FileSystemSettings,
    about: About,
}

impl SettingsDialog {
    pub fn new(target_media_type: &TargetMediaType, sub_dir_depth_limit: u8) -> Self {
        Self {
            tab: Tab::default(),
            general_settings: GeneralSettings::new(target_media_type, sub_dir_depth_limit),
            ai_settings: AiSettings::default(),
            file_system_settings: FileSystemSettings::default(),
            about: About::default(),
        }
    }
}
