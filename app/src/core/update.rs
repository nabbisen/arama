mod cache;
mod component;
mod ui;

use iced::Task;

use super::{App, message::Message};

impl App {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            // --- navigation ---
            Message::NavTo(page) => self.handle_nav_to(page),

            // --- cache pipeline ---
            Message::CacheRequire(target) => self.handle_cache_require(target),
            Message::ThumbnailCacheFinished(ret) => self.handle_thumbnail_cache_finished(ret),
            Message::EmbeddingCacheFinished(err) => self.handle_embedding_cache_finished(err),
            Message::CachePageMessage(message) => self.handle_cache_page_message(message),
            Message::CacheClearFinished(result) => self.handle_cache_clear_finished(result),

            // --- component delegation ---
            Message::SetupMessage(message) => self.handle_setup_message(message),
            Message::GalleryMessage(message) => self.handle_gallery_message(message),
            Message::HeaderMessage(message) => self.handle_header_message(message),
            Message::AsideMessage(message) => self.handle_aside_message(message),
            Message::FooterMessage(message) => self.handle_footer_message(message),
            Message::MediaFocusDialogMessage(message) => {
                self.handle_media_focus_dialog_message(message)
            }
            Message::SimilarPairsDialogMessage(message) => {
                self.handle_similar_pairs_dialog_message(message)
            }
            Message::SettingsDialogMessage(message) => self.handle_settings_dialog_message(message),

            // --- ui housekeeping ---
            Message::ContextMenuMessage(message) => self.handle_context_menu_message(message),
            Message::DialogClose => self.handle_dialog_close(),
            Message::CloseMenus => self.handle_close_menus(),
            Message::ToastDismiss(id) => self.handle_toast_dismiss(id),
            Message::ToastSweep => self.handle_toast_sweep(),
            Message::CursorMove(point) => self.handle_cursor_move(point),
        }
    }
}
