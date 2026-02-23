use super::thumbnail_size_slider;

#[derive(Debug, Clone)]
pub enum Message {
    IncludeImage(bool),
    IncludeVideo(bool),
    ThumbnailSizeSliderMessage(thumbnail_size_slider::message::Message),
    SimilarPairsOpen,
}
