use iced::{
    Element,
    Length::Fill,
    widget::{column, container, row},
};

pub mod gallery;

use super::{App, message::Message};

impl App {
    pub fn view(&self) -> Element<'_, Message> {
        let gallery = self
            .gallery
            .view()
            .map(|message| Message::GalleryMessage(message));

        let header = self.header.view().map(Message::HeaderMessage);
        let aside = self.aside.view().map(Message::AsideMessage);
        let footer = self.footer.view().map(Message::FooterMessage);

        column![
            container(header).height(60),
            container(row![aside, gallery]).height(Fill),
            container(footer).height(40)
        ]
        .into()
    }
}
