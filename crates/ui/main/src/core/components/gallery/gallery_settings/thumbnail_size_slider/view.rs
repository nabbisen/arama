use arama_env::MAX_THUMBNAIL_SIZE;
use arama_env::MIN_THUMBNAIL_SIZE;
use iced::Element;
use iced::widget::slider;

use super::SLIDER_STEP;

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
