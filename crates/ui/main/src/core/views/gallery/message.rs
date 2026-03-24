use crate::{components::gallery::image_cell, core::components::gallery::gallery_settings};

#[derive(Debug, Clone)]
pub enum Message {
    GallerySettingsMessage(gallery_settings::message::Message),
    ImageCellMessage(image_cell::message::Message),
    SimilarPairsOpen,
}
