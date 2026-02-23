use super::MediaFocusDialog;
use super::{message::Message, output::Output};

impl MediaFocusDialog {
    pub fn update(&mut self, message: Message) -> Output {
        match message {
            Message::CloseClick => Output::CloseClick,
        }
    }
}
