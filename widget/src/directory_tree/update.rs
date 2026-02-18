use iced::Task;

use super::DirectoryTree;
use super::message::Message;

impl DirectoryTree {
    // update 関数内での処理例
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::FileNodeMessage(message) => {
                let _ = self.root.update(message);
                Task::none()
            }
        }
    }
}
