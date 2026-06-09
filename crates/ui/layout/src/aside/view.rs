use iced::{Element, Length, widget::{column, container}};

use super::{
    Aside,
    message::{Internal, Message},
};

impl Aside {
    pub fn view(&self) -> Element<'_, Message> {
        let dir_tree = container(
            self.dir_tree
                .view()
                .map(|x| Message::Internal(Internal::DirTreeMessage(x))),
        )
        .padding([5, 0]);

        column![dir_tree]
            .width(Length::Shrink)
            .spacing(5)
            .into()
    }
}
