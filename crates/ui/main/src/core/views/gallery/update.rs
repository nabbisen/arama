use iced::Task;

use crate::core::components::gallery::gallery_settings;

use super::{
    Gallery,
    message::Message,
    // subscription::Input
};
// use crate::core::settings::Settings;
// use arama_embedding::store::file::file_embedding_map::FileEmbeddingMap;

impl Gallery {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::GallerySettingsMessage(message) => {
                let _ = self.gallery_settings.update(message.clone());

                match message {
                    gallery_settings::message::Message::SimilarPairsOpen => {
                        return Task::done(Message::SimilarPairsOpen);
                    }
                    _ => (),
                }

                Task::none()
            }
            Message::ImageCellMessage(_message) => Task::none(),
            Message::SimilarPairsOpen => Task::none(),
        }
    }
}
