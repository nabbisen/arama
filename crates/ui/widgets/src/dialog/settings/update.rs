use super::{Settings, message::Message, output::Output};

impl Settings {
    pub fn update(&mut self, message: Message) -> Option<Output> {
        match message {
            Message::TabSelect(tab) => self.tab = tab,
            Message::GeneralSettingsTabMessage(_message) => (),
            Message::AiSettingsTabMessage(_message) => (),
            Message::FileSystemSettingsTabMessage(_message) => (),
        }
        None
    }
}
