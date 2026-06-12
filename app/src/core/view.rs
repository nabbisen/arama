use arama_ui_main::views::setup;
use iced::{
    Element,
    Length::Fill,
    widget::{button, column, container, mouse_area, row, text, tooltip},
};
use arama_i18n::t;
use lucide_icons::iced::{icon_database, icon_folder, icon_settings};
use snora::{AppLayout, Dialog as SnoraDialog, ToastPosition, render};

use super::{App, Dialog, NavPage, message::Message};

impl App {
    pub fn view(&self) -> Element<'_, Message> {
        // Setup screen: bypass the main skeleton entirely.
        if !self.setup.finished && !setup::util::ready() {
            return self.setup.view().map(Message::SetupMessage);
        }

        // ── Side-bar nav rail ─────────────────────────────────────────
        let side_bar: Element<Message> = {
            let explorer = tooltip(
                button(icon_folder())
                    .style(if self.nav_page == NavPage::Explorer {
                        button::primary
                    } else {
                        button::text
                    })
                    .on_press(Message::NavTo(NavPage::Explorer)),
                text(t("nav.explorer")),
                tooltip::Position::Right,
            );

            let cache = tooltip(
                button(icon_database())
                    .style(if self.nav_page == NavPage::Cache {
                        button::primary
                    } else {
                        button::text
                    })
                    .on_press(Message::NavTo(NavPage::Cache)),
                text(t("nav.cache")),
                tooltip::Position::Right,
            );

            let settings = tooltip(
                button(icon_settings())
                    .style(if self.nav_page == NavPage::Settings {
                        button::primary
                    } else {
                        button::text
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
                // Header (dir input + action buttons) spans the full
                // width of the Explorer page.
                let header = self.header.view().map(Message::HeaderMessage);

                // Tiling row: always-visible dir tree + gallery.
                let aside = self.aside.view().map(Message::AsideMessage);
                let gallery = self
                    .gallery
                    .view(self.footer.thumbnail_size())
                    .map(Message::GalleryMessage);

                let tiling = mouse_area(
                    container(row![aside, gallery])
                        .height(Fill)
                        .padding([0, 20]),
                )
                .on_move(Message::CursorMove);

                column![header, tiling].into()
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

        let mut layout: AppLayout<Element<'_, Message>, Message> =
            AppLayout::new(body)
                .side_bar(side_bar)
                .footer(footer)
                .on_close_menus(Message::CloseMenus)
                .on_close_modals(Message::DialogClose)
                .toasts(self.toasts.clone())
                .toast_position(ToastPosition::BottomEnd);

        // Context menu: only populate when open so snora's backdrop is
        // active only then.
        if self.context_menu.is_open() {
            layout = layout.context_menu(
                self.context_menu.view().map(Message::ContextMenuMessage),
            );
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

        render(layout)
    }
}
