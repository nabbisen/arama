use arama_ui_main::views::setup;
use arama_ui_widgets::dialog::overlay;
use iced::{
    Element,
    Length::Fill,
    widget::{column, container, mouse_area, row, stack},
};

use super::{App, Dialog, message::Message};

impl App {
    pub fn view(&self) -> Element<'_, Message> {
        if !self.setup.finished && !setup::util::ready() {
            return self.setup.view().map(Message::SetupMessage).into();
        }

        let content = self
            .workbench
            .view(self.footer.thumbnail_size())
            .map(|message| Message::WorkbenchMessage(message));

        let header = self.header.view().map(Message::HeaderMessage);
        let aside = self.aside.view().map(Message::AsideMessage);
        let footer = self.footer.view().map(Message::FooterMessage);

        let layout = mouse_area(column![
            container(header).height(60),
            container(row![aside, content])
                .height(Fill)
                .padding([0, 20]),
            container(footer).height(40)
        ])
        .on_move(Message::CursorMove);

        let context_menu = self.context_menu.view().map(Message::ContextMenuMessage);

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
