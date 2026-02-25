use arama_ui_widgets::dialog::overlay;
use iced::{
    Element,
    Length::Fill,
    widget::{button, column, container, mouse_area, row, space, stack},
};

use super::{App, ContextMenu, Dialog, message::Message};

impl App {
    pub fn view(&self) -> Element<'_, Message> {
        let content = self
            .gallery
            .view()
            .map(|message| Message::GalleryMessage(message));

        let header = self.header.view().map(Message::HeaderMessage);
        let aside = self.aside.view().map(Message::AsideMessage);
        let footer = self.footer.view().map(Message::FooterMessage);

        let layout = mouse_area(column![
            container(header).height(60),
            container(row![aside, content]).height(Fill),
            container(footer).height(40)
        ])
        .on_move(Message::CursorMove);

        let context_menu = match &self.context_menu {
            ContextMenu::ImageCell(path) => container(
                button("file manager").on_press(Message::FileManagerShow(path.to_path_buf())),
            )
            .padding([self.context_menu_point.y, self.context_menu_point.x]),
            ContextMenu::None => container(space()),
        };

        let layout_with_context_menu = stack!(layout, context_menu);

        let dialog = match &self.dialog {
            Some(Dialog::MediaFocusDialog(x)) => {
                Some(x.view().map(Message::MediaFocusDialogMessage))
            }
            Some(Dialog::SimilarPairsDialog(x)) => {
                Some(x.view().map(Message::SimilarPairsDialogMessage))
            }
            Some(Dialog::SettingsDialog(x)) => Some(x.view().map(Message::SettingsDialogMessage)),
            None => None,
        };

        overlay(
            layout_with_context_menu.into(),
            dialog,
            Some(Message::DialogClose),
        )
        .into()
    }
}
