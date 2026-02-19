pub mod message;
pub mod update;
pub mod view;

const DEFAULT_THUMBNAIL_SIZE: u32 = 224;
const MIN_THUMBNAIL_SIZE: u32 = 64;
const MAX_THUMBNAIL_SIZE: u32 = 448;
const SLIDER_STEP: u32 = 32;

pub struct ThumbnailSizeSlider {
    pub value: u32,
}

impl Default for ThumbnailSizeSlider {
    fn default() -> Self {
        Self {
            value: DEFAULT_THUMBNAIL_SIZE,
        }
    }
}
