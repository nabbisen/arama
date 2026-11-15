use std::{path::PathBuf, sync::Arc};

use arama_sidecar::media::video::video_engine::discovery::{
    FfmpegDiscoveryEvent, FfmpegDiscoveryTicket,
};
use arama_ui_layout::{aside, footer, header};
use arama_ui_main::views::{cache_page, gallery, setup};
use arama_ui_widgets::{
    context_menu,
    dialog::{confirm_dialog, media_focus_dialog, settings_dialog, similar_pairs_dialog},
};
use iced::Point;
use snora::ToastAttention;

use super::NavPage;

/// T069: named so the `Message` variant and `run_thumbnail_phase`'s own
/// return type (`app/src/core/update/cache.rs`) stay exactly in sync
/// without clippy's `type_complexity` lint on the nested
/// `Result<Vec<(_, _)>, _>` at either site. `Err` means the cache could
/// not even be opened for writing - distinguished from an empty
/// `Ok(vec![])` ("nothing to index"), which the two used to collapse
/// into indistinguishably.
pub type ThumbnailPhaseResult = Result<Vec<(PathBuf, Arc<arama_cache::Result<()>>)>, String>;

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
    ThumbnailCacheFinished(ThumbnailPhaseResult),
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
    ConfirmDialogMessage(confirm_dialog::message::Message),
    /// Task 039: async cache-contents delete finished, or the error from
    /// the first entry that failed to remove.
    CacheDeleteFinished(Result<(), String>),
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
    /// T070 (snora 0.52 -> 0.57, RFC-113/RFC-120): the reader's pointer or
    /// keyboard focus reached or left a toast - forwarded to
    /// `self.holds.attend(...)`, which is what makes a toast wait while
    /// someone is still reading it instead of expiring mid-sentence.
    ToastAttention(u64, ToastAttention),
    /// RFC 047 S6 / review 161 #3: a non-capturing subscription tick
    /// (`subscription()`) - it carries no state itself, since
    /// `Subscription::map`'s closure must not capture anything.
    /// `handle_similar_pairs_progress_tick` reads the live
    /// `ScanControl` snapshot and forwards it into the dialog, the same
    /// pattern `ToastSweep` already uses for reading `self.toasts`
    /// inside `update` rather than inside the subscription closure.
    SimilarPairsProgressTick,
    CursorMove(Point),
    /// RFC 044: every key press, routed through `snora::keyboard::cycle_zones`
    /// in `update` (its sibling `dismiss_on_escape` was removed in snora
    /// 0.57 - snora's own engine has handled Escape since 0.55, via
    /// `on_close_modals`/`on_close_menus` in `view.rs`) - this
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
