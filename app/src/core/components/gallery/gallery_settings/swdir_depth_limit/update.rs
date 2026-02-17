use super::SwdirDepthLimit;
use super::message::Message;

impl SwdirDepthLimit {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::ValueChanged(value) => {
                let digit_chars = value
                    .chars()
                    .filter(|c| c.is_ascii_digit())
                    .collect::<String>();
                self.value = if let Ok(number) = digit_chars.parse::<usize>() {
                    Some(number)
                } else {
                    None
                };
            }
        }
    }
}
