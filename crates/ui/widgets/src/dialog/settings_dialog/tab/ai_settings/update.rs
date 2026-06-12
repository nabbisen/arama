use arama_ai::model::{model_container::clip, model_manager::ModelManager};
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
                        let clip_model_manager = match ModelManager::new(clip::model()) {
                            Ok(x) => x,
                            Err(err) => return Some(err.to_string()),
                        };
                        match clip_model_manager.ensure().await {
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
                    async { VideoEngine::download_and_install().await.err().map(|e| e.to_string()) },
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
