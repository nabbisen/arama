use std::time::Instant;

use iced::Task;

use super::{DOUBLE_CLICK_INTERVAL_MILLIS, file_node};

use super::DirTree;
use super::message::Message;

impl DirTree {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::FileNodeMessage(file_node_message) => {
                let _ = self.root.update(file_node_message.clone());

                match file_node_message {
                    file_node::message::Message::DirClick(path) => {
                        let now = Instant::now();

                        let (last_path, last_time) = match self.dir_last_clicked.clone() {
                            Some(dir_last_clicked) => dir_last_clicked,
                            None => {
                                self.dir_last_clicked = Some((path.to_owned(), now));
                                return Task::done(Message::DirClick(path));
                            }
                        };

                        if last_path == path
                            && now.duration_since(last_time) <= DOUBLE_CLICK_INTERVAL_MILLIS
                        {
                            self.dir_last_clicked = None;
                            return Task::done(Message::DirDoubleClick(path));
                        }

                        self.dir_last_clicked = None;
                        Task::done(Message::DirClick(path))
                    }
                    _ => Task::none(),
                }
            }
            _ => Task::none(),
        }
    }
}
