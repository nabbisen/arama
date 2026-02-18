use iced::Element;

use super::DirectoryTree;
use super::message::Message;

impl DirectoryTree {
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
