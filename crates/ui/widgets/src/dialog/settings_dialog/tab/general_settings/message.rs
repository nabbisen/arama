use arama_env::ThemePreset;
use arama_env::target_media_type::TargetMediaType;
use arama_i18n::Locale;

#[derive(Debug, Clone)]
pub enum Message {
    TargetMediaTypeChanged(TargetMediaType),
    SubDirDepthLimitChanged(u8),
    SimilarityThresholdChanged(f32),
    LocaleChanged(Locale),
    ThemeChanged(ThemePreset),
}
