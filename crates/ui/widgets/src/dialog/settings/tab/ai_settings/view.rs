use arama_ai::model::clip;
use iced::{
    Element,
    widget::{button, column, container, space, text},
};

use super::{AiSettings, message::Message};

impl AiSettings {
    pub fn view(&self) -> Element<'_, Message> {
        // todo
        if clip::model().ready().unwrap_or(false) {
            return text("AI model is ready.").into();
        }

        column![
            text("AI model for image analysis is not found.\nShould get model from huggingface.co. Network will be used this time only"),
            button("Load").on_press(Message::LoadStart),
            if !self.message.is_empty() { container(text(self.message.to_owned())) } else { container(space()) }
        ]
        .into()
    }
}
