use arama_ai::model::model_container::clip;
use arama_i18n::t;
use arama_sidecar::media::video::video_engine::VideoEngine;
use iced::Task;

use super::{AiSettings, message::Message};

impl AiSettings {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::LoadStart => {
                self.message = t("settings.ai.clip_loading");

                Task::perform(
                    async {
                        // ensure_safetensors converts the downloaded PyTorch model to
                        // SafeTensors format if necessary; it is synchronous.
                        match clip::model().ensure_safetensors() {
                            Ok(_) => None,
                            Err(err) => Some(err.to_string()),
                        }
                    },
                    Message::Loaded,
                )
            }
            Message::Loaded(result) => {
                if let Some(err) = result {
                    self.message = err;
                }
                Task::none()
            }
            Message::GetFfmpegStart => {
                self.message = t("settings.ai.ffmpeg_fetching");
                Task::perform(
                    async {
                        VideoEngine::download_and_install()
                            .await
                            .err()
                            .map(|e| e.to_string())
                    },
                    Message::FfmpegGot,
                )
            }
            Message::FfmpegGot(result) => {
                self.message = match result {
                    None => t("settings.ai.ffmpeg_ready"),
                    Some(err) => err,
                };
                Task::none()
            }
        }
    }
}
