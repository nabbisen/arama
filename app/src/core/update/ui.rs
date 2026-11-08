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
        // RFC 047 S3.3: the similar-pairs scan must be interruptible, and
        // closing its dialog is the only control surface it has for
        // that - there is no separate Cancel button. A `None` handle
        // (no scan in flight, or a different dialog was open) makes
        // this a no-op.
        if let Some(handle) = self.similar_pairs_task_handle.take() {
            handle.abort();
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
        snora::toast::sweep_expired(&mut self.toasts, std::time::Instant::now());
        Task::none()
    }

    pub(super) fn handle_cursor_move(&mut self, point: iced::Point) -> Task<Message> {
        if let ContextMenuState::None = self.context_menu.state {
            self.context_menu.update_point(point)
        };
        Task::none()
    }
}
