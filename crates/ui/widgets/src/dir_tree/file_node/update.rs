use std::fs;
use std::path::PathBuf;

use iced::Task;

use super::{
    FileNode,
    message::{Internal, Message},
    util::is_hidden,
};

impl FileNode {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Event(_) => return Task::none(),
            Message::Internal(message) => {
                match message {
                    Internal::TreeLoaded(file_node) => {
                        self.name = file_node.name;
                        self.path = file_node.path;
                        self.is_dir = file_node.is_dir;
                        self.is_expanded = file_node.is_expanded;
                        self.children = file_node.children;
                    }
                    Internal::ToggleExpand((path, include_file, include_hidden)) => {
                        // 再帰的にツリーを更新して is_expanded を切り替える
                        self.update_tree_lazy(&path, include_file, include_hidden);
                    }
                }
                Task::none()
            }
        }
    }

    fn update_tree_lazy(&mut self, path: &PathBuf, include_file: bool, include_hidden: bool) {
        if self.path == *path {
            self.is_expanded = !self.is_expanded;

            if self.is_expanded {
                if let Ok(entries) = fs::read_dir(&self.path) {
                    self.children.clear();

                    for entry in entries.flatten() {
                        if !include_file {
                            if let Ok(file_type) = entry.file_type() {
                                if !file_type.is_dir() {
                                    continue;
                                }
                            }
                        }

                        if !include_hidden {
                            if is_hidden(&entry.path()) {
                                continue;
                            }
                        }

                        self.children
                            .push(FileNode::new(entry.path(), false, false)); // 子のさらに下は読み込まない
                    }

                    self.children
                        .sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then(a.name.cmp(&b.name)));
                }
            }
            return;
        }

        for child in &mut self.children {
            child.update_tree_lazy(path, include_file, include_hidden);
        }
    }
}
