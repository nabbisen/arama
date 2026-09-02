use arama_env::ThemePreset;
use arama_env::target_media_type::TargetMediaType;
use arama_i18n::Locale;

use super::{
    Tab,
    tab::{about, ai_settings, file_system_settings, general_settings},
};

#[derive(Debug, Clone)]
pub enum Message {
    TargetMediaTypeChanged(TargetMediaType),
    SubDirDepthLimitChanged(u8),
    SimilarityThresholdChanged(f32),
    LocaleChanged(Locale),
    ThemeChanged(ThemePreset),
    RefreshAiCapabilities,
    FfmpegRecheckRequested,
    FfmpegSelectRequested,
    FfmpegClearRequested,
    /// Task 039: bubbled up so the app can open the confirmation dialog -
    /// the tab itself never deletes anything.
    CacheDeleteRequested,
    TabSelect(Tab),
    GeneralSettingsTabMessage(general_settings::message::Message),
    AiSettingsTabMessage(ai_settings::message::Message),
    FileSystemSettingsTabMessage(file_system_settings::message::Message),
    AboutTabMessage(about::message::Message),
}
