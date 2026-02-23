use super::MediaFocus;
use super::{message::Message, output::Output};

impl MediaFocus {
    pub fn update(&mut self, message: Message) -> Output {
        match message {
            Message::CloseClick => Output::CloseClick,
        }
    }
}
