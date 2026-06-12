use iced::Task;

use super::{GeneralSettings, message::Message};

impl GeneralSettings {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::TargetMediaTypeChanged(x) => {
                self.target_media_type = x;
            }
            Message::SubDirDepthLimitChanged(value) => self.sub_dir_depth_limit = value,
            Message::SimilarityThresholdChanged(v) => self.similarity_threshold = v,
            Message::LocaleChanged(l) => self.locale = l,
        }
        Task::none()
    }
}
