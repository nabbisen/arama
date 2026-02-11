use iced::Element;
use iced::widget::text_input;

use super::SwdirDepthLimit;
use super::message::Message;

impl SwdirDepthLimit {
    pub fn view(&self) -> Element<'_, Message> {
        let text_input_value = if let Some(value) = self.value {
            value.to_string()
        } else {
            String::new()
        };

        text_input(
            "Deepest subdirectory level to scan",
            text_input_value.as_str(),
        )
        .on_input(Message::ValueChanged)
        .into()
    }
}
