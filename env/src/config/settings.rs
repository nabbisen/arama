use serde::{Deserialize, Serialize};

pub mod target_media_type;

use target_media_type::TargetMediaType;

#[derive(Serialize, Deserialize, Default)]
pub struct Settings {
    pub root_dir_path: String,
    pub target_media_type: TargetMediaType,
    pub sub_dir_depth_limit: u8,
}
