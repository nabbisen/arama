use arama_env::MAX_THUMBNAIL_SIZE;
use arama_env::MIN_THUMBNAIL_SIZE;
use iced::Element;
use iced::widget::row;
use iced::widget::slider;
use iced::widget::text;

use super::SLIDER_STEP;

use super::ThumbnailSizeSlider;
use super::message::Message;

impl ThumbnailSizeSlider {
    pub fn view(&self) -> Element<'_, Message> {
        row![
            text("Thumbnail size"),
            slider(
                MIN_THUMBNAIL_SIZE..=MAX_THUMBNAIL_SIZE,
                self.value,
                Message::ValueChanged,
            )
            .step(SLIDER_STEP)
        ]
        .spacing(10)
        .into()
    }
}
