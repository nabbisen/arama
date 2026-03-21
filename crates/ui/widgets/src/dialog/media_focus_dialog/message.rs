use super::types::SimilarMediaItem;

#[derive(Debug, Clone)]
pub enum Message {
    SimilarMediaReady(Vec<SimilarMediaItem>),
    MediaItemEnter(String),
    MediaItemExit,
    ViewSizeToggle(bool),
    CloseClick,
}
