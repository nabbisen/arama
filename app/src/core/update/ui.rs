use arama_ui_widgets::context_menu::ContextMenuState;
use iced::Task;

use super::super::{App, NavPage, message::Message};

impl App {
    pub(super) fn handle_toggle_aside(&mut self) -> Task<Message> {
        self.aside_open = !self.aside_open;
        // When opening, expand the tree to the current directory so
        // the user immediately sees their location in the hierarchy.
        if self.aside_open {
            let current = std::path::PathBuf::from(&self.settings.root_dir_path);
            let expand_task = self.aside.expand_to(&current);
            return expand_task.map(Message::AsideMessage);
        }
        Task::none()
    }

    pub(super) fn handle_nav_to(&mut self, page: NavPage) -> Task<Message> {
        let reload = if page == NavPage::Cache {
            self.cache_page.load_task().map(Message::CachePageMessage)
        } else {
            Task::none()
        };
        self.nav_page = page;
        reload
    }

    pub(super) fn handle_context_menu_message(
        &mut self,
        message: arama_ui_widgets::context_menu::message::Message,
    ) -> Task<Message> {
        self.context_menu
            .update(message)
            .map(Message::ContextMenuMessage)
    }

    pub(super) fn handle_dialog_close(&mut self) -> Task<Message> {
        self.dialog = None;
        // Review 161 #2: the similar-pairs scan must actually be
        // interruptible, and closing its dialog is the only control
        // surface it has for that - there is no separate Cancel button.
        // `cancel()` on the shared `ScanControl`, not a `Task::abortable`
        // `Handle` (which cannot stop a `spawn_blocking` closure once it
        // is running - see `ScanControl`'s own doc comment). A `None`
        // scan (none in flight, or a different dialog was open) makes
        // this a no-op.
        if let Some(control) = self.similar_pairs_scan.take() {
            control.cancel();
        }
        Task::none()
    }

    pub(super) fn handle_close_menus(&mut self) -> Task<Message> {
        self.context_menu.state = ContextMenuState::None;
        Task::none()
    }

    pub(super) fn handle_toast_dismiss(&mut self, id: u64) -> Task<Message> {
        self.toasts.retain(|t| t.id != id);
        Task::none()
    }

    pub(super) fn handle_toast_sweep(&mut self) -> Task<Message> {
        // T070 (snora 0.52 -> 0.57): `snora::toast::sweep_expired` is
        // removed in 0.57 - `Holds::sweep` is its replacement, and also
        // respects whatever toast the reader is currently attending to
        // (RFC-113/RFC-120), which the free function never knew about.
        self.holds
            .sweep(&mut self.toasts, std::time::Instant::now());
        Task::none()
    }

    /// T070 (snora 0.52 -> 0.57, RFC-113/RFC-120): the end state, not the
    /// deprecated `on_toast_hover`/`ToastHover`/`Holds::hover` trio (removed
    /// in 0.59.0) - a toast now waits while the reader's pointer *or*
    /// keyboard focus is on it, not only the pointer.
    pub(super) fn handle_toast_attention(
        &mut self,
        id: u64,
        attention: snora::ToastAttention,
    ) -> Task<Message> {
        self.holds
            .attend(id, attention, &mut self.toasts, std::time::Instant::now());
        Task::none()
    }

    pub(super) fn handle_cursor_move(&mut self, point: iced::Point) -> Task<Message> {
        if let ContextMenuState::None = self.context_menu.state {
            self.context_menu.update_point(point)
        };
        Task::none()
    }
}
