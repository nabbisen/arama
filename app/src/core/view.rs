use arama_ui_main::views::setup;
use iced::{
    Element,
    Length::Fill,
    widget::{container, mouse_area, row},
};
use snora::{AppLayout, Dialog as SnoraDialog, ToastPosition, render};

use super::{App, Dialog, message::Message};

impl App {
    pub fn view(&self) -> Element<'_, Message> {
        // Setup screen: bypass the main skeleton entirely.
        if !self.setup.finished && !setup::util::ready() {
            return self.setup.view().map(Message::SetupMessage);
        }

        // ---- body: aside rail + gallery, with horizontal padding --------
        let aside = self.aside.view().map(Message::AsideMessage);
        let gallery = self
            .gallery
            .view(self.footer.thumbnail_size())
            .map(Message::GalleryMessage);

        // CursorMove is only needed to track context-menu position over
        // gallery cells, so wrapping the body row is sufficient.
        let body = mouse_area(
            container(row![aside, gallery])
                .height(Fill)
                .padding([0, 20]),
        )
        .on_move(Message::CursorMove);

        // ---- slots: header and footer own their own slot heights -----------
        let header = self.header.view().map(Message::HeaderMessage);
        let footer = self.footer.view().map(Message::FooterMessage);

        // ---- AppLayout: build the layered skeleton ----------------------
        let mut layout: AppLayout<Element<'_, Message>, Message> = AppLayout::new(body.into())
            .header(header)
            .footer(footer)
            .on_close_menus(Message::CloseMenus)
            .on_close_modals(Message::DialogClose)
            .toasts(self.toasts.clone())
            .toast_position(ToastPosition::BottomEnd);

        // Context menu: only populate the slot when open so snora's
        // transparent backdrop and dismissal are active only then.
        if self.context_menu.is_open() {
            layout = layout.context_menu(
                self.context_menu.view().map(Message::ContextMenuMessage),
            );
        }

        // Modal dialogs: map the active dialog variant to an Element and
        // hand it to snora for centered, dimmed presentation.
        if let Some(dialog) = &self.dialog {
            let elem: Element<'_, Message> = match dialog {
                Dialog::MediaFocusDialog(x) => x.view().map(Message::MediaFocusDialogMessage),
                Dialog::SimilarPairsDialog(x) => x.view().map(Message::SimilarPairsDialogMessage),
                Dialog::SettingsDialog(x) => x.view().map(Message::SettingsDialogMessage),
            };
            layout = layout.dialog(SnoraDialog::new(elem));
        }

        render(layout)
    }
}
