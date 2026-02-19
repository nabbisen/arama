use iced::Element;
use iced::widget::slider;

use super::{MAX_THUMBNAIL_SIZE, MIN_THUMBNAIL_SIZE, SLIDER_STEP};

use super::ThumbnailSizeSlider;
use super::message::Message;

impl ThumbnailSizeSlider {
    pub fn view(&self) -> Element<'_, Message> {
        slider(
            MIN_THUMBNAIL_SIZE..=MAX_THUMBNAIL_SIZE,
            self.value,
            Message::ValueChanged,
        )
        .step(SLIDER_STEP)
        .into()
    }
}
