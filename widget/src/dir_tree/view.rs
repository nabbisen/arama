use iced::Element;

use super::DirTree;
use super::message::Message;

impl DirTree {
    pub fn view(&self) -> Element<'static, Message> {
        self.root
            .view(
                &self.selected_path,
                0,
                self.include_file,
                self.include_hidden,
            )
            .map(Message::FileNodeMessage)
    }
}
