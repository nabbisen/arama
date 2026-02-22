use arama_widget::{aside, dialog, footer, header};

use super::gallery;

#[derive(Debug, Clone)]
pub enum Message {
    GalleryMessage(gallery::message::Message),
    HeaderMessage(header::message::Message),
    AsideMessage(aside::message::Message),
    FooterMessage(footer::message::Message),
    SettingsDialogMessage(dialog::settings::message::Message),
    DialogClose,
}
