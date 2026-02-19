use super::{GallerySettings, message::Message};

impl GallerySettings {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::ThumbnailSizeSliderMessage(message) => {
                let _ = self.thumbnail_size.update(message);
            }
        }
    }
}
