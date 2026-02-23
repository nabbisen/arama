use arama_ai::{ModelManager, model::clip};
use iced::Task;

use super::{AiSettings, message::Message};

impl AiSettings {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::LoadStart => {
                self.message = "loading...".to_owned();

                Task::perform(
                    async {
                        let clip_model_manager = match ModelManager::new(clip::model()) {
                            Ok(x) => x,
                            Err(err) => return Some(err.to_string()),
                        };
                        match clip_model_manager.get_safetensors_from_pytorch().await {
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
        }
    }
}
