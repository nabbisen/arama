use arama_ui_main::views::setup;
use arama_ui_widgets::dialog::overlay;
use iced::{
    Element,
    Length::Fill,
    widget::{button, column, container, mouse_area, row, space, stack, text},
};

use super::{App, ContextMenu, Dialog, message::Message};

impl App {
    pub fn view(&self) -> Element<'_, Message> {
        if !self.setup.finished && !setup::util::ready() {
            return self.setup.view().map(Message::SetupMessage).into();
        }

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
            ContextMenu::ImageCell(path) => container(column![
                space().height(self.context_menu_point.y),
                row![
                    space().width(self.context_menu_point.x),
                    column![
                        container(text(
                            path.canonicalize()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .to_string()
                        ))
                        .style(container::secondary),
                        button("file manager")
                            .on_press(Message::FileManagerShow(path.to_path_buf())),
                    ]
                    .width(self.gallery.gallery_settings.thumbnail_size() as f32)
                    .spacing(5)
                ]
            ]),
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
