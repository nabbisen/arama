pub mod message;
pub mod update;
pub mod view;

const DEFAULT_THUMBNAIL_SIZE: u16 = 224;
const MIN_THUMBNAIL_SIZE: u16 = 128;
pub const MAX_THUMBNAIL_SIZE: u16 = 384;
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
