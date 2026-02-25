use std::{ffi::OsStr, path::Path};

use arama_env::{IMAGE_EXTENSION_ALLOWLIST, VIDEO_EXTENSION_ALLOWLIST};

pub mod video;

pub enum MediaType {
    Image,
    Video,
    Other,
}

impl MediaType {
    pub fn inspect(path: &Path) -> Self {
        let extension = path
            .extension()
            .unwrap_or(OsStr::new(""))
            .to_str()
            .unwrap_or("");
        if IMAGE_EXTENSION_ALLOWLIST.contains(&extension) {
            Self::Image
        } else if VIDEO_EXTENSION_ALLOWLIST.contains(&extension) {
            Self::Video
        } else {
            Self::Other
        }
    }
}
