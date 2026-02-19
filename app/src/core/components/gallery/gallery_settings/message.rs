use super::thumbnail_size_slider;

#[derive(Debug, Clone)]
pub enum Message {
    ThumbnailSizeSliderMessage(thumbnail_size_slider::message::Message),
}
