use arama_env::MAX_THUMBNAIL_SIZE;
use arama_env::MIN_THUMBNAIL_SIZE;
use arama_i18n::t;
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
            text(t("footer.thumbnail_size")),
            // Fixed rather than the default `Fill`: this row now sits under
            // a `Shrink`-sized ancestor (T055, footer/view.rs) so its size
            // comes from its content, not the other way round. A `Fill`
            // child has no content size to contribute, so the slider would
            // resolve to ~0 width instead of actually reserving room for
            // itself.
            slider(
                MIN_THUMBNAIL_SIZE..=MAX_THUMBNAIL_SIZE,
                self.value,
                Message::ValueChanged,
            )
            .step(SLIDER_STEP)
            .width(120.0)
        ]
        .spacing(10)
        .into()
    }
}
