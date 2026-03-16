use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TargetMediaType {
    pub include_image: bool,
    pub include_video: bool,
}

impl Default for TargetMediaType {
    fn default() -> Self {
        Self {
            include_image: true,
            include_video: false,
        }
    }
}
