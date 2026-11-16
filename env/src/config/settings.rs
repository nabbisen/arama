use arama_i18n::Locale;
use serde::{Deserialize, Serialize};

pub mod cache_lookup_strategy;
pub mod ffmpeg_location;
pub mod target_media_type;

use crate::{DEFAULT_THUMBNAIL_SIZE, MIN_IMAGE_SIMILARITY};
use cache_lookup_strategy::CacheLookupStrategy;
use ffmpeg_location::FfmpegLocationPreference;
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

/// User-selectable text size (RFC 052 §2). Scales the whole
/// `snora_design::Typography` role set by one factor, derived once at
/// startup - not a runtime-switchable global like [`ThemePreset`], since
/// `iced::Settings.default_text_size` (the route 118 of 120 `text(`
/// sites depend on) is read once when the application starts and never
/// again (RFC 043 §3.1). Pure data, same reasoning as `ThemePreset`
/// above: no GUI dependency, so it lives in `arama-env`; the scaling
/// itself lives in `arama-theme`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextScale {
    #[default]
    Default,
    Large,
    Larger,
}

impl TextScale {
    /// All steps in display order.
    pub fn all() -> &'static [TextScale] {
        &[TextScale::Default, TextScale::Large, TextScale::Larger]
    }

    /// The multiplier applied to every `snora_design::TextRole::size` in
    /// the active preset's typography scale. `line_height` is left
    /// untouched - it is already a multiplier relative to `size`
    /// (`snora_design::TextRole`'s own doc comment), so it stays
    /// proportionally correct without being scaled a second time.
    ///
    /// Chosen, not derived from a formula: `Default` 1.0 (16px body,
    /// unchanged), `Large` 1.25 (20px body), `Larger` 1.5 (24px body) -
    /// three steps a reader can tell apart at a glance, with `Larger`
    /// substantial enough to matter for the audience RFC 052 names
    /// without doubling every dimension in the UI.
    pub fn factor(self) -> f32 {
        match self {
            TextScale::Default => 1.0,
            TextScale::Large => 1.25,
            TextScale::Larger => 1.5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    #[serde(default)]
    pub text_scale: TextScale,
    #[serde(default)]
    pub ffmpeg_location: FfmpegLocationPreference,
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
            text_scale: TextScale::default(),
            ffmpeg_location: FfmpegLocationPreference::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{TextScale, ThemePreset};

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

    #[test]
    fn text_scale_factors_and_default() {
        assert_eq!(TextScale::default(), TextScale::Default);
        assert_eq!(TextScale::all().len(), 3);
        assert_eq!(TextScale::Default.factor(), 1.0);
        assert_eq!(TextScale::Large.factor(), 1.25);
        assert_eq!(TextScale::Larger.factor(), 1.5);
        // The resulting body px per step, stated plainly (handoff §13):
        // 16.0, 20.0, 24.0.
        assert_eq!(16.0 * TextScale::Default.factor(), 16.0);
        assert_eq!(16.0 * TextScale::Large.factor(), 20.0);
        assert_eq!(16.0 * TextScale::Larger.factor(), 24.0);
    }

    #[test]
    fn text_scale_serde_round_trip() {
        for (scale, json) in [
            (TextScale::Default, "\"default\""),
            (TextScale::Large, "\"large\""),
            (TextScale::Larger, "\"larger\""),
        ] {
            let encoded = serde_json::to_string(&scale).unwrap();
            assert_eq!(encoded, json);
            let decoded: TextScale = serde_json::from_str(json).unwrap();
            assert_eq!(decoded, scale);
        }
    }

    /// Handoff §10: an unknown value must not error, it must fall back
    /// to `Default` - matching the existing precedent every other
    /// persisted setting enum here already follows (none has
    /// `#[serde(other)]`; an unrecognized variant fails the *whole*
    /// file's parse, which `App::new()` already recovers from by
    /// falling back to `Settings::default()` and showing a startup
    /// notice, RFC 017's Fatal-startup tier) - not a new per-field
    /// mechanism `text_scale` would need on its own.
    #[test]
    fn text_scale_unknown_value_fails_to_parse_like_every_other_setting_enum() {
        let err = serde_json::from_str::<TextScale>("\"huge\"").unwrap_err();
        assert!(err.to_string().contains("unknown variant"));
    }
}
