use iced::Element;

use super::{Aside, message::Message};

impl Aside {
    pub fn view(&self) -> Element<'static, Message> {
        self.dir_tree.view().map(Message::DirTreeMessage).into()
    }
}
