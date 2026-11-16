use arama_env::target_media_type::TargetMediaType;
use arama_env::{TextScale, ThemePreset};
use arama_i18n::Locale;

pub mod message;
mod update;
mod view;

#[derive(Clone, Debug)]
pub struct GeneralSettings {
    target_media_type: TargetMediaType,
    sub_dir_depth_limit: u8,
    similarity_threshold: f32,
    locale: Locale,
    theme: ThemePreset,
    text_scale: TextScale,
}

impl GeneralSettings {
    pub fn new(
        target_media_type: &TargetMediaType,
        sub_dir_depth_limit: u8,
        similarity_threshold: f32,
        locale: Locale,
        theme: ThemePreset,
        text_scale: TextScale,
    ) -> Self {
        Self {
            target_media_type: target_media_type.to_owned(),
            sub_dir_depth_limit,
            similarity_threshold,
            locale,
            theme,
            text_scale,
        }
    }
}
