use super::{dir_nav, settings};

#[derive(Debug, Clone)]
pub enum Message {
    DirNavMessage(dir_nav::message::Message),
    SettingsMessage(settings::message::Message),
}
