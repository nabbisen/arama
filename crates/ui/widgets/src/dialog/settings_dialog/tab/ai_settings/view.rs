use arama_ai::model::model_container::clip;
use arama_cache::{FfmpegStatus, VideoEngine};
use iced::{
    Element,
    widget::{button, column, container, space, text},
};

use super::{AiSettings, message::Message};

impl AiSettings {
    pub fn view(&self) -> Element<'_, Message> {
        // todo
        let clip: Element<Message> = if clip::model().ready().unwrap_or(false) {
            text("AI model is ready.").into()
        } else {
            column![
                text(
                    "AI model for image analysis is not found.\nShould get model from huggingface.co. Network will be used"
                ),
                button("Load").on_press(Message::LoadStart),
            ].into()
        };

        let ffmpeg: Element<Message> = if VideoEngine::ready() != FfmpegStatus::NotExists {
            text("ffmpeg is ready.").into()
        } else {
            column![
                text(
                    "ffmpeg for video analysis is not found.\nShould get executable. Network will be used"
                ),
                // todo: on_press()
                button("Get"),
            ].into()
        };

        let message = if !self.message.is_empty() {
            container(text(self.message.to_owned()))
        } else {
            container(space())
        };

        column![clip, ffmpeg, message].into()
    }
}
