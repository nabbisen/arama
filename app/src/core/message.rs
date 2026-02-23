use arama_ui_layout::{aside, footer, header};
use arama_ui_main::views::gallery;
use arama_ui_widgets::dialog;

#[derive(Debug, Clone)]
pub enum Message {
    GalleryMessage(gallery::message::Message),
    HeaderMessage(header::message::Message),
    AsideMessage(aside::message::Message),
    FooterMessage(footer::message::Message),
    MediaFocusDialogMessage(dialog::media_focus::message::Message),
    SimilarPairsDialogMessage(dialog::similar_pairs::message::Message),
    SettingsDialogMessage(dialog::settings::message::Message),
    DialogClose,
}
