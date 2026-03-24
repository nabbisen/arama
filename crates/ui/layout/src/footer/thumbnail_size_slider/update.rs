use super::ThumbnailSizeSlider;
use super::message::Message;

impl ThumbnailSizeSlider {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::ValueChanged(value) => {
                self.value = value;
            }
        }
    }
}
