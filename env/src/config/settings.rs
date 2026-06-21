use arama_i18n::Locale;
use serde::{Deserialize, Serialize};

pub mod cache_lookup_strategy;
pub mod target_media_type;

use crate::{DEFAULT_THUMBNAIL_SIZE, MIN_IMAGE_SIMILARITY};
use cache_lookup_strategy::CacheLookupStrategy;
use target_media_type::TargetMediaType;

/// User-selectable application theme preset. Maps to the four Snora Design
/// token presets (and, for the base iced theme, to `Theme::Light` / `Dark`).
/// Pure data — no GUI dependency, so it lives in `arama-env` alongside the
/// other persisted setting enums. The token / iced-`Theme` mapping lives in
/// `arama-theme`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThemePreset {
    #[default]
    Light,
    Dark,
    HighContrastLight,
    HighContrastDark,
}

impl ThemePreset {
    /// All presets in display order.
    pub fn all() -> &'static [ThemePreset] {
        &[
            ThemePreset::Light,
            ThemePreset::Dark,
            ThemePreset::HighContrastLight,
            ThemePreset::HighContrastDark,
        ]
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    pub root_dir_path: String,
    pub target_media_type: TargetMediaType,
    pub sub_dir_depth_limit: u8,
    pub thumbnail_size: u16,
    pub cache_lookup_strategy: CacheLookupStrategy,
    /// Cosine-similarity threshold for the focus view and similarity pairs
    /// finder. Defaults to [`MIN_IMAGE_SIMILARITY`] (0.86).
    /// `serde(default)` ensures existing settings files load cleanly.
    #[serde(default = "default_similarity_threshold")]
    pub similarity_threshold: f32,
    #[serde(default)]
    pub locale: Locale,
    #[serde(default)]
    pub theme: ThemePreset,
}

fn default_similarity_threshold() -> f32 {
    MIN_IMAGE_SIMILARITY
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            root_dir_path: String::default(),
            target_media_type: TargetMediaType::default(),
            sub_dir_depth_limit: 0,
            thumbnail_size: DEFAULT_THUMBNAIL_SIZE,
            cache_lookup_strategy: CacheLookupStrategy::default(),
            similarity_threshold: default_similarity_threshold(),
            locale: Locale::default(),
            theme: ThemePreset::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ThemePreset;

    #[test]
    fn theme_preset_discriminants() {
        // The u8 discriminants are relied on by arama-theme's atomic global
        // (THEME_ID encode/decode), so pin them here.
        assert_eq!(ThemePreset::Light as u8, 0);
        assert_eq!(ThemePreset::Dark as u8, 1);
        assert_eq!(ThemePreset::HighContrastLight as u8, 2);
        assert_eq!(ThemePreset::HighContrastDark as u8, 3);
        assert_eq!(ThemePreset::all().len(), 4);
        assert_eq!(ThemePreset::default(), ThemePreset::Light);
    }

    #[test]
    fn theme_preset_serde_round_trip() {
        for (preset, json) in [
            (ThemePreset::Light, "\"light\""),
            (ThemePreset::Dark, "\"dark\""),
            (ThemePreset::HighContrastLight, "\"high_contrast_light\""),
            (ThemePreset::HighContrastDark, "\"high_contrast_dark\""),
        ] {
            let encoded = serde_json::to_string(&preset).unwrap();
            assert_eq!(encoded, json);
            let decoded: ThemePreset = serde_json::from_str(json).unwrap();
            assert_eq!(decoded, preset);
        }
    }
}
