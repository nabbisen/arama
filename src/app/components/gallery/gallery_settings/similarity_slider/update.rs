use super::message::Message;
use super::{SimilaritySlider, similarity_quality::SimilarityQuality};

impl SimilaritySlider {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::ValueChanged(value) => {
                self.similarity_quality = SimilarityQuality::from_index(value);
            }
        }
    }
}
