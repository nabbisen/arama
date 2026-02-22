use iced::Element;

use super::{Aside, message::Message};

impl Aside {
    pub fn view(&self) -> Element<'_, Message> {
        self.dir_tree.view().map(Message::DirTreeMessage).into()
    }
}
