use super::{GallerySettings, message::Message, output::Output};

impl GallerySettings {
    pub fn update(&mut self, message: Message) -> Option<Output> {
        match message {
            Message::IncludeImage(b) => {
                self.target_media_type.include_image = b;
                return Some(Output::TargetMediaTypeChange(
                    self.target_media_type.clone(),
                ));
            }
            Message::IncludeVideo(b) => {
                self.target_media_type.include_video = b;
                return Some(Output::TargetMediaTypeChange(
                    self.target_media_type.clone(),
                ));
            }
            Message::ThumbnailSizeSliderMessage(message) => {
                let _ = self.thumbnail_size.update(message);
            }
            Message::SimilarPairsOpen => (),
        }
        None
    }
}
