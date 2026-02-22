use arama_widget::dialog::overlay;
use iced::{
    Element,
    Length::Fill,
    widget::{column, container, row},
};

pub mod gallery;

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
            Some(Dialog::MediaFocus(x)) => {
                // state.view() が返す Element<settings::Msg> を Element<Msg> に変換
                Some(x.view().map(Message::MediaFocusDialogMessage))
            }
            Some(Dialog::Settings(x)) => {
                // state.view() が返す Element<settings::Msg> を Element<Msg> に変換
                Some(x.view().map(Message::SettingsDialogMessage))
            }
            None => None,
        };

        overlay(layout.into(), dialog, Some(Message::DialogClose)).into()
    }
}
