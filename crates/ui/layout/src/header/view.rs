use iced::{
    Element,
    widget::{button, row},
};
use lucide_icons::iced::icon_group;

use super::{Header, message::Message};

impl Header {
    pub fn view(&self) -> Element<'_, Message> {
        let similar_pairs_button = button(icon_group()).on_press_maybe(if self.embedding_cached {
            Some(Message::SimilarPairsDialogOpen)
        } else {
            None
        });

        row![
            self.dir_nav.view().map(Message::DirNavMessage),
            similar_pairs_button,
            self.settings_nav.view().map(Message::SettingsNavMessage)
        ]
        .padding(10)
        .into()
    }
}
