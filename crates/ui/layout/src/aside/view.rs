use iced::{Element, Length, widget::{column, container}};

use super::{
    Aside,
    message::{Internal, Message},
};

impl Aside {
    pub fn view(&self) -> Element<'_, Message> {
        let tree = container(
            self.tree
                .view(|e| Message::Internal(Internal::TreeEvent(e))),
        )
        .padding([5, 0]);

        column![tree]
            .width(Length::Shrink)
            .spacing(5)
            .into()
    }
}
