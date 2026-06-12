use arama_i18n::Locale;
use serde::{Deserialize, Serialize};

pub mod cache_lookup_strategy;
pub mod target_media_type;

use crate::{DEFAULT_THUMBNAIL_SIZE, MIN_IMAGE_SIMILARITY};
use cache_lookup_strategy::CacheLookupStrategy;
use target_media_type::TargetMediaType;

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
        }
    }
}
