use super::Settings;
use super::{message::Message, output::Output};

impl Settings {
    pub fn update(&mut self, message: Message) -> Output {
        match message {}
    }
}
