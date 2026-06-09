use iced::{
    Element,
    widget::{button, container, row},
};
use lucide_icons::iced::icon_group;

use super::{
    Header,
    message::{Event, Internal, Message},
};

impl Header {
    pub fn view(&self) -> Element<'_, Message> {
        let similar_pairs_button = button(icon_group()).on_press_maybe(if self.embedding_cached {
            Some(Message::Event(Event::SimilarPairsDialogOpen))
        } else {
            None
        });

        let inner = row![
            self.dir_nav
                .view()
                .map(|x| Message::Internal(Internal::DirNavMessage(x))),
            similar_pairs_button,
        ]
        .spacing(20)
        .padding(10);

        container(inner).height(60).into()
    }
}
