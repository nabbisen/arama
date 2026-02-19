pub mod message;
pub mod update;
pub mod view;

const DEFAULT_THUMBNAIL_SIZE: u16 = 224;
const MIN_THUMBNAIL_SIZE: u16 = 64;
const MAX_THUMBNAIL_SIZE: u16 = 448;
const SLIDER_STEP: u16 = 32;

pub struct ThumbnailSizeSlider {
    pub value: u16,
}

impl Default for ThumbnailSizeSlider {
    fn default() -> Self {
        Self {
            value: DEFAULT_THUMBNAIL_SIZE,
        }
    }
}
