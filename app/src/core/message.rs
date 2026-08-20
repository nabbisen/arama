use std::{path::PathBuf, sync::Arc};

use arama_sidecar::media::video::video_engine::discovery::{
    FfmpegDiscoveryEvent, FfmpegDiscoveryTicket,
};
use arama_ui_layout::{aside, footer, header};
use arama_ui_main::views::{cache_page, gallery, setup};
use arama_ui_widgets::{
    context_menu,
    dialog::{media_focus_dialog, settings_dialog, similar_pairs_dialog},
};
use iced::Point;

use super::NavPage;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FfmpegRequestIntent {
    Startup,
    Recheck,
    Selection,
    ClearToAuto,
}

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
    FfmpegDiscoveryEvent {
        epoch: u64,
        ticket: FfmpegDiscoveryTicket,
        event: Option<FfmpegDiscoveryEvent>,
    },
    FfmpegDirectoryPicked {
        picker_epoch: u64,
        directory: Option<PathBuf>,
    },
    ContextMenuMessage(context_menu::message::Message),
    ToggleAside,
    DialogClose,
    CloseMenus,
    ToastDismiss(u64),
    ToastSweep,
    CursorMove(Point),
    /// RFC 044: every key press, routed through `snora::keyboard`'s pure
    /// helpers (`dismiss_on_escape`, `cycle_zones`) in `update` - this
    /// variant carries the raw event rather than a pre-decided intent so
    /// arama installs exactly one keyboard subscription for everything
    /// this RFC adds.
    KeyPressed(iced::keyboard::Key, iced::keyboard::Modifiers),
    /// The keyboard subscription's fallback for events this RFC does not
    /// use (`KeyReleased`, `ModifiersChanged`) - `Subscription::map`
    /// requires producing a `Message` for every event on the stream, and
    /// reusing an unrelated variant for "nothing happened" would be
    /// misleading at every call site that matches on it.
    NoOp,
}
