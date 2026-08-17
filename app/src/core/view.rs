use arama_i18n::t;
use iced::{
    Element,
    Length::Fill,
    widget::{button, column, container, mouse_area, row, text, tooltip},
};
use lucide_icons::iced::{
    icon_database, icon_folder, icon_panel_left_close, icon_panel_left_open, icon_settings,
};
use snora::{AppLayout, Dialog as SnoraDialog, ToastPosition, design::render};

#[cfg(test)]
use super::ARAMA_DATA_HOME_TEST_LOCK;
use super::{App, Dialog, NavPage, message::Message, setup_complete};

impl App {
    pub fn view(&self) -> Element<'_, Message> {
        // RFC 041, RFC 017 Fatal-startup tier: no toast layer, no
        // AppLayout skeleton, nothing dismissible - arama has nowhere to
        // persist anything, so nothing behind this message is meaningful
        // to show. This is the first real implementation of this tier;
        // every previous "startup error" in this codebase was a
        // Recoverable-action toast (StartupNotice::error), dismissible
        // and overlaid on an otherwise-normal, otherwise-usable window.
        if let Some(message) = &self.fatal_startup_error {
            return fatal_startup_view(message);
        }

        // Setup screen: bypass the main skeleton, but keep the shared
        // toast layer so startup notices remain visible before setup
        // finishes.
        if !setup_complete(self.setup.finished, self.setup.ready()) {
            let setup = self.setup.view().map(Message::SetupMessage);
            return render(self.toast_layout(setup), &arama_theme::tokens());
        }

        // ── Side-bar nav rail ─────────────────────────────────────────
        let side_bar: Element<Message> = {
            let explorer = tooltip(
                button(icon_folder())
                    .style(if self.nav_page == NavPage::Explorer {
                        arama_theme::primary
                    } else {
                        arama_theme::ghost
                    })
                    .on_press(Message::NavTo(NavPage::Explorer)),
                text(t("nav.explorer")),
                tooltip::Position::Right,
            );

            let cache = tooltip(
                button(icon_database())
                    .style(if self.nav_page == NavPage::Cache {
                        arama_theme::primary
                    } else {
                        arama_theme::ghost
                    })
                    .on_press(Message::NavTo(NavPage::Cache)),
                text(t("nav.cache")),
                tooltip::Position::Right,
            );

            let settings = tooltip(
                button(icon_settings())
                    .style(if self.nav_page == NavPage::Settings {
                        arama_theme::primary
                    } else {
                        arama_theme::ghost
                    })
                    .on_press(Message::NavTo(NavPage::Settings)),
                text(t("nav.settings")),
                tooltip::Position::Right,
            );

            column![explorer, cache, settings]
                .spacing(4)
                .padding(8)
                .into()
        };

        // ── Page body ─────────────────────────────────────────────────
        let body: Element<Message> = match self.nav_page {
            NavPage::Explorer => {
                // Toggle button: opens/closes the aside tree pane.
                let toggle_icon = if self.aside_open {
                    icon_panel_left_close()
                } else {
                    icon_panel_left_open()
                };
                let toggle_tooltip = if self.aside_open {
                    t("aside.toggle.close")
                } else {
                    t("aside.toggle.open")
                };
                let toggle = tooltip(
                    button(toggle_icon)
                        .style(if self.aside_open {
                            arama_theme::primary
                        } else {
                            arama_theme::ghost
                        })
                        .on_press(Message::ToggleAside),
                    text(toggle_tooltip),
                    tooltip::Position::Right,
                );

                // Header row: toggle + dir input + action buttons.
                let header_row = row![toggle, self.header.view().map(Message::HeaderMessage),]
                    .spacing(4)
                    .align_y(iced::Alignment::Center);

                // Tiling row: optional tree pane + gallery.
                let gallery = self
                    .gallery
                    .view(self.footer.thumbnail_size())
                    .map(Message::GalleryMessage);

                let content: iced::Element<Message> = if self.aside_open {
                    let aside = self.aside.view().map(Message::AsideMessage);
                    row![aside, gallery].into()
                } else {
                    gallery
                };

                let tiling = mouse_area(container(content).height(Fill).padding([0, 20]))
                    .on_move(Message::CursorMove);

                column![header_row, tiling].into()
            }
            NavPage::Cache => self.cache_page.view().map(Message::CachePageMessage),
            NavPage::Settings => container(
                self.settings_page
                    .view()
                    .map(Message::SettingsDialogMessage),
            )
            .padding(20)
            .into(),
        };

        // ── AppLayout skeleton ────────────────────────────────────────
        let footer = self.footer.view().map(Message::FooterMessage);

        let mut layout = self
            .toast_layout(body)
            .side_bar(side_bar)
            .footer(footer)
            .on_close_menus(Message::CloseMenus)
            .on_close_modals(Message::DialogClose);

        // Context menu: only populate when open so snora's backdrop is
        // active only then.
        if self.context_menu.is_open() {
            layout = layout.context_menu(self.context_menu.view().map(Message::ContextMenuMessage));
        }

        // Modal dialogs (MediaFocus, SimilarPairs only — Settings is a
        // page now).
        if let Some(dialog) = &self.dialog {
            let elem: Element<'_, Message> = match dialog {
                Dialog::MediaFocusDialog(x) => x.view().map(Message::MediaFocusDialogMessage),
                Dialog::SimilarPairsDialog(x) => x.view().map(Message::SimilarPairsDialogMessage),
            };
            layout = layout.dialog(SnoraDialog::new(elem));
        }

        render(layout, &arama_theme::tokens())
    }

    fn toast_layout<'a>(
        &'a self,
        body: Element<'a, Message>,
    ) -> AppLayout<Element<'a, Message>, Message> {
        AppLayout::new(body)
            .toasts(self.toasts.clone())
            .toast_position(ToastPosition::BottomEnd)
    }
}

/// RFC 041, RFC 017 Fatal-startup tier: the whole window, nothing else -
/// no side bar, no toast layer, no dismiss affordance. `message` is the
/// specific resolution/creation failure (which location, which error),
/// appended below the fixed, translated explanation so a user can act on
/// it (or report it) without a debugger.
fn fatal_startup_view(message: &str) -> Element<'_, Message> {
    use iced::widget::{center, space};

    center(
        column![
            text(t("startup.fatal_error.title")).size(20),
            space().height(12),
            text(t("startup.fatal_error.body")),
            space().height(12),
            text(message.to_owned()),
        ]
        .max_width(560)
        .spacing(4),
    )
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::widget::text;
    use snora::{Toast, ToastIntent};

    #[test]
    fn setup_screen_layout_carries_startup_toasts() {
        // RFC 041: App::new() now resolves and creates real data
        // locations. Without this override it would touch this machine's
        // actual platform config/data/cache directories - isolate to a
        // scratch dir instead, restoring any ambient value afterward so
        // this test does not leak state to others in the same binary.
        //
        // Task 023: `ARAMA_DATA_HOME` is process-global - hold the shared
        // lock for the mutation window so this can never interleave with
        // `core::tests`'s own `ARAMA_DATA_HOME`-mutating tests.
        let _guard = ARAMA_DATA_HOME_TEST_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let previous = std::env::var_os(arama_env::DATA_HOME_ENV_VAR);
        let scratch = std::env::temp_dir().join(format!(
            "arama-view-test-{}-setup-screen-toasts",
            std::process::id()
        ));
        unsafe {
            std::env::set_var(arama_env::DATA_HOME_ENV_VAR, &scratch);
        }

        let mut app = App::new().0;

        unsafe {
            match &previous {
                Some(value) => std::env::set_var(arama_env::DATA_HOME_ENV_VAR, value),
                None => std::env::remove_var(arama_env::DATA_HOME_ENV_VAR),
            }
        }
        let _ = std::fs::remove_dir_all(&scratch);
        let existing_toasts = app.toasts.len();
        app.toasts.push(Toast::new(
            99,
            ToastIntent::Warning,
            "Startup warning",
            "visible on setup",
            Message::ToastDismiss(99),
        ));

        let layout = app.toast_layout(text("setup").into());

        assert_eq!(layout.toasts.len(), existing_toasts + 1);
        assert_eq!(layout.toast_position, ToastPosition::BottomEnd);
        assert!(layout.side_bar.is_none());
        assert!(layout.footer.is_none());
    }
}
