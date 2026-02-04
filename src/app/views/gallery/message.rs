use std::path::PathBuf;

use crate::app::components::gallery::menus;

#[derive(Debug, Clone)]
pub enum Message {
    ImagesLoaded(Vec<PathBuf>),
    MenusMessage(menus::message::Message),
}
