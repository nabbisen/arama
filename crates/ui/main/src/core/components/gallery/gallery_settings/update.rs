use iced::Task;

use super::{GallerySettings, message::Message};

impl GallerySettings {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::TargetMediaTypeChanged(target_media_type) => {
                self.target_media_type = target_media_type;
            }
            Message::SubDirDepthLimitChanged(value) => self.sub_dir_depth_limit = value,
            Message::ThumbnailSizeSliderMessage(message) => {
                let _ = self.thumbnail_size.update(message);
            }
            Message::SimilarPairsOpen => (),
        }
        Task::none()
    }
}
