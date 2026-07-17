pub mod message;
mod update;
mod view;

mod tab;

use arama_env::target_media_type::TargetMediaType;
use arama_env::{ThemePreset, ffmpeg_location::FfmpegLocationPreference};
use arama_i18n::Locale;
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
    pub fn new(
        target_media_type: &TargetMediaType,
        sub_dir_depth_limit: u8,
        similarity_threshold: f32,
        locale: Locale,
        theme: ThemePreset,
        ffmpeg_preference: FfmpegLocationPreference,
    ) -> Self {
        Self {
            tab: Tab::default(),
            general_settings: GeneralSettings::new(
                target_media_type,
                sub_dir_depth_limit,
                similarity_threshold,
                locale,
                theme,
            ),
            ai_settings: AiSettings::new(ffmpeg_preference),
            file_system_settings: FileSystemSettings::default(),
            about: About::default(),
        }
    }

    pub fn set_ffmpeg_checking(&mut self, preference: FfmpegLocationPreference) {
        self.ai_settings.set_ffmpeg_checking(preference);
    }

    pub fn set_ffmpeg_outcome(
        &mut self,
        preference: FfmpegLocationPreference,
        outcome: &arama_sidecar::media::video::video_engine::discovery::FfmpegDiscoveryOutcome,
    ) {
        self.ai_settings.set_ffmpeg_outcome(preference, outcome);
    }

    pub fn set_ffmpeg_ready(&mut self, preference: FfmpegLocationPreference, ready: bool) {
        self.ai_settings.set_ffmpeg_ready(preference, ready);
    }

    pub fn set_ffmpeg_candidate_failure(
        &mut self,
        preference: &FfmpegLocationPreference,
        outcome: &arama_sidecar::media::video::video_engine::discovery::FfmpegDiscoveryOutcome,
    ) {
        self.ai_settings
            .set_ffmpeg_candidate_failure(preference, outcome);
    }

    pub fn set_ffmpeg_candidate_checking(&mut self, preference: &FfmpegLocationPreference) {
        self.ai_settings.set_ffmpeg_candidate_checking(preference);
    }

    pub fn set_ffmpeg_draining(&mut self, preference: FfmpegLocationPreference) {
        self.ai_settings.set_ffmpeg_draining(preference);
    }

    pub fn set_ffmpeg_select_enabled(&mut self, enabled: bool) {
        self.ai_settings.set_ffmpeg_select_enabled(enabled);
    }
}
