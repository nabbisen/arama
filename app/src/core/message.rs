use std::{path::PathBuf, sync::Arc};

use arama_ui_layout::{aside, footer, header};
use arama_ui_main::views::{gallery, setup};
use arama_ui_widgets::dialog::{media_focus_dialog, settings_dialog, similar_pairs_dialog};
use iced::Point;

#[derive(Debug, Clone)]
pub enum Message {
    CacheRequire,
    ThumbnailCacheFinished(Vec<(PathBuf, Arc<arama_cache::Result<()>>)>),
    EmbeddingCacheFinished(Option<String>),
    SetupMessage(setup::message::Message),
    GalleryMessage(gallery::message::Message),
    HeaderMessage(header::message::Message),
    AsideMessage(aside::message::Message),
    FooterMessage(footer::message::Message),
    MediaFocusDialogMessage(media_focus_dialog::message::Message),
    SimilarPairsDialogMessage(similar_pairs_dialog::message::Message),
    SettingsDialogMessage(settings_dialog::message::Message),
    FileManagerShow(PathBuf),
    DialogClose,
    CursorMove(Point),
}
