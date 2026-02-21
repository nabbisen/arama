use arama_widget::{aside, footer, header};

use super::gallery;

pub enum Message {
    GalleryMessage(gallery::message::Message),
    HeaderMessage(header::message::Message),
    AsideMessage(aside::message::Message),
    FooterMessage(footer::message::Message),
}
