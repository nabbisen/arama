pub mod message;
mod update;
pub mod view;

const SLIDER_STEP: u16 = 32;

#[derive(Clone, Debug)]
pub struct ThumbnailSizeSlider {
    pub value: u16,
}

impl ThumbnailSizeSlider {
    pub fn new(thumbnail_size: u16) -> Self {
        Self {
            value: thumbnail_size,
        }
    }
}
