use std::time::Instant;

use iced::Task;

use crate::directory_tree::{DOUBLE_CLICK_INTERVAL_MILLIS, file_node};

use super::DirectoryTree;
use super::message::Message;

impl DirectoryTree {
    // update 関数内での処理例
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::FileNodeMessage(file_node_message) => {
                let _ = self.root.update(file_node_message.clone());

                match file_node_message {
                    file_node::message::Message::DirectoryClick(path) => {
                        let now = Instant::now();

                        let (last_path, last_time) = match self.directory_last_clicked.clone() {
                            Some(directory_last_clicked) => directory_last_clicked,
                            None => {
                                self.directory_last_clicked = Some((path, now));
                                return Task::none();
                            }
                        };

                        if last_path == path
                            && now.duration_since(last_time) <= DOUBLE_CLICK_INTERVAL_MILLIS
                        {
                            self.directory_last_clicked = None;
                            return Task::done(Message::DirectoryDoubleClick(path.to_owned()));
                        }

                        self.directory_last_clicked = None;
                        Task::none()
                    }
                    _ => Task::none(),
                }
            }
            _ => Task::none(),
        }
    }
}
