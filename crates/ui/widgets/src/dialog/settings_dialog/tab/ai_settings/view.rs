use arama_ai::model::model_container::clip;
use arama_i18n::t;
use arama_sidecar::media::video::video_engine::{FfmpegStatus, VideoEngine};
use iced::{
    Element,
    widget::{button, column, container, space, text},
};

use super::{AiSettings, message::Message};

impl AiSettings {
    pub fn view(&self) -> Element<'_, Message> {
        let clip: Element<Message> = if clip::model().ready().unwrap_or(false) {
            text(t("settings.ai.clip_ready")).into()
        } else {
            column![
                text(t("settings.ai.clip_missing")),
                button(text(t("settings.ai.clip_load"))).on_press(Message::LoadStart),
            ]
            .into()
        };

        let ffmpeg: Element<Message> = if VideoEngine::ready() != FfmpegStatus::NotExists {
            text(t("settings.ai.ffmpeg_ready")).into()
        } else {
            column![
                text(t("settings.ai.ffmpeg_missing")),
                button(text(t("settings.ai.ffmpeg_get"))).on_press(Message::GetFfmpegStart),
            ]
            .into()
        };

        let message = if !self.message.is_empty() {
            container(text(self.message.to_owned()))
        } else {
            container(space())
        };

        column![clip, ffmpeg, message].into()
    }
}
