use super::MediaFocusDialog;
use super::{message::Message, output::Output};

impl MediaFocusDialog {
    pub fn update(&mut self, message: Message) -> Option<Output> {
        match message {
            Message::ViewSizeToggle(actual_size) => self.actual_size = actual_size,
            Message::CloseClick => return Some(Output::CloseClick),
        }
        None
    }
}
