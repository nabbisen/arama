use arama_env::DEFAULT_THUMBNAIL_SIZE;

pub mod message;
mod update;
pub mod view;

const SLIDER_STEP: u16 = 32;

#[derive(Clone, Debug)]
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
