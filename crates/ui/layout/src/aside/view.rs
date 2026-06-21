use iced::{
    Element, Length,
    widget::{Id, column, container, scrollable},
};

use super::{
    Aside, SCROLLABLE_ID,
    message::{Internal, Message},
};

impl Aside {
    pub fn view(&self) -> Element<'_, Message> {
        // Wrap in a named scrollable so `finish_expand` can snap
        // the viewport to the selected row via `operation::snap_to`.
        let tree = scrollable(
            container(
                self.tree
                    .view(|e| Message::Internal(Internal::TreeEvent(e))),
            )
            .padding([5, 0]),
        )
        .id(Id::new(SCROLLABLE_ID))
        .width(Length::Fill)
        .height(Length::Fill);

        column![tree]
            .width(Length::FillPortion(1))
            .height(Length::Fill)
            .spacing(5)
            .into()
    }
}
