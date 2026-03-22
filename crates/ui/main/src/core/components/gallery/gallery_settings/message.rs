use arama_env::target_media_type::TargetMediaType;

use super::thumbnail_size_slider;

#[derive(Debug, Clone)]
pub enum Message {
    TargetMediaTypeChanged(TargetMediaType),
    SubDirDepthLimitChanged(u8),
    ThumbnailSizeSliderMessage(thumbnail_size_slider::message::Message),
    SimilarPairsOpen,
}
