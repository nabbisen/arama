use arama_embedding::{ModelManager, model::clip};
use iced::{
    Element, Task,
    widget::{button, column, container, space, text},
};

#[derive(Clone)]
pub enum Message {
    LoadStart,
    Loaded(Option<String>),
}

#[derive(Default)]
pub struct ModelLoader {
    message: String,
}

impl ModelLoader {
    pub fn view(&self) -> Element<'_, Message> {
        column![
            text("AI model for image analysis is not found.\nShould get model from huggingface.co. Network will be used this time only"),
            button("Load").on_press(Message::LoadStart),
            if !self.message.is_empty() { container(text(self.message.to_owned())) } else { container(space()) }
        ]
        .into()
    }

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
