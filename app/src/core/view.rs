use arama_ui_widgets::dialog::overlay;
use iced::{
    Element,
    Length::Fill,
    widget::{column, container, row},
};

use super::{App, Dialog, message::Message};

impl App {
    pub fn view(&self) -> Element<'_, Message> {
        let content = self
            .gallery
            .view()
            .map(|message| Message::GalleryMessage(message));

        let header = self.header.view().map(Message::HeaderMessage);
        let aside = self.aside.view().map(Message::AsideMessage);
        let footer = self.footer.view().map(Message::FooterMessage);

        let layout = column![
            container(header).height(60),
            container(row![aside, content]).height(Fill),
            container(footer).height(40)
        ];

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

        overlay(layout.into(), dialog, Some(Message::DialogClose)).into()
    }
}
