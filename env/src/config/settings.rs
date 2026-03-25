use serde::{Deserialize, Serialize};

pub mod target_media_type;

use crate::DEFAULT_THUMBNAIL_SIZE;
use target_media_type::TargetMediaType;

#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    pub root_dir_path: String,
    pub target_media_type: TargetMediaType,
    pub sub_dir_depth_limit: u8,
    pub thumbnail_size: u16,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            root_dir_path: String::default(),
            target_media_type: TargetMediaType::default(),
            sub_dir_depth_limit: 0,
            thumbnail_size: DEFAULT_THUMBNAIL_SIZE,
        }
    }
}
