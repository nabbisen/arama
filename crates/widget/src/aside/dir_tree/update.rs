use std::time::Instant;

use super::{DOUBLE_CLICK_INTERVAL_MILLIS, file_node};

use super::{DirTree, message::Message, output::Output};

impl DirTree {
    pub fn update(&mut self, message: Message) -> Option<Output> {
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
                                return Some(Output::DirClick(path));
                            }
                        };

                        if last_path == path
                            && now.duration_since(last_time) <= DOUBLE_CLICK_INTERVAL_MILLIS
                        {
                            self.dir_last_clicked = None;
                            return Some(Output::DirDoubleClick(path));
                        }

                        self.dir_last_clicked = None;
                        return Some(Output::DirClick(path));
                    }
                    _ => (),
                }
            }
            _ => (),
        }
        None
    }
}
