use super::{GallerySettings, message::Message};

impl GallerySettings {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::SwdirDepthLimitMessage(message) => {
                let _ = self.swdir_depth_limit.update(message);
            }
            Message::SimilaritySliderMessage(message) => {
                let _ = self.similarity_slider.update(message);
            }
        }
    }
}
