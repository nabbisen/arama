use iced::{
    Element, Task,
    widget::{button, column, container, space, text},
};

use std::path::PathBuf;

#[derive(Clone)]
pub enum Message {
    LoadStart,
    Loaded(Result<PathBuf, String>),
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
                        let response = reqwest::get(crate::app::URL).await;
                        match response {
                            Ok(response) => {
                                let bytes =
                                    response.bytes().await.expect("failed to get model bytes");
                                let path = crate::app::PYTORCH_MODEL;
                                match tokio::fs::write(path, &bytes).await {
                                    Ok(_) => Ok(PathBuf::from(path)),
                                    Err(err) => Err(err.to_string()),
                                }
                            }
                            Err(err) => Err(err.to_string()),
                        }
                    },
                    Message::Loaded,
                )
            }
            Message::Loaded(path) => {
                self.message = String::default();

                let path = if let Ok(path) = path {
                    path
                } else {
                    return Task::none();
                };

                pt2safetensors::Pt2Safetensors::default()
                    .removes_pt_at_conversion_success()
                    .convert(&path, &crate::engine::SAFETENSORS_MODEL.into())
                    .expect("failed to convert pytorch to safetensors");

                Task::none()
            }
        }
    }
}
