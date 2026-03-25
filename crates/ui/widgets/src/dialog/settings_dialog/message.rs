use arama_env::target_media_type::TargetMediaType;

use super::{
    Tab,
    tab::{about, ai_settings, file_system_settings, general_settings},
};

#[derive(Debug, Clone)]
pub enum Message {
    TargetMediaTypeChanged(TargetMediaType),
    TabSelect(Tab),
    GeneralSettingsTabMessage(general_settings::message::Message),
    AiSettingsTabMessage(ai_settings::message::Message),
    FileSystemSettingsTabMessage(file_system_settings::message::Message),
    AboutTabMessage(about::message::Message),
}
