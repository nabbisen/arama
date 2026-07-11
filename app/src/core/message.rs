use std::{path::PathBuf, sync::Arc};

use arama_ui_layout::{aside, footer, header};
use arama_ui_main::views::{cache_page, gallery, setup};
use arama_ui_widgets::{
    context_menu,
    dialog::{media_focus_dialog, settings_dialog, similar_pairs_dialog},
};
use iced::Point;

use super::NavPage;

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone)]
pub enum Message {
    NavTo(NavPage),
    /// Start the indexing pipeline. `None` targets the Explorer's
    /// current directory; `Some` targets an explicit directory tree
    /// (Cache page requests).
    CacheRequire(Option<swdir::DirNode>),
    CachePageMessage(cache_page::message::Message),
    /// Async per-row cache clear finished: removed count or error.
    CacheClearFinished(Result<usize, String>),
    /// Async cache prune finished: report or error.
    CachePruneFinished(Result<arama_cache::CachePruneReport, String>),
    ThumbnailCacheFinished(Vec<(PathBuf, Arc<arama_cache::Result<()>>)>),
    EmbeddingCacheFinished(
        Result<arama_ai::pipeline::encode::image::embeddings::EmbeddingRunReport, String>,
    ),
    SetupMessage(setup::message::Message),
    GalleryMessage(gallery::message::Message),
    HeaderMessage(header::message::Message),
    AsideMessage(aside::message::Message),
    FooterMessage(footer::message::Message),
    MediaFocusDialogMessage(media_focus_dialog::message::Message),
    SimilarPairsDialogMessage(similar_pairs_dialog::message::Message),
    SettingsDialogMessage(settings_dialog::message::Message),
    ContextMenuMessage(context_menu::message::Message),
    ToggleAside,
    DialogClose,
    CloseMenus,
    ToastDismiss(u64),
    ToastSweep,
    CursorMove(Point),
}
