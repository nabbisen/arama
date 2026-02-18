use std::fs;
use std::path::PathBuf;

use iced::Task;

use super::FileNode;
use super::message::Message;

impl FileNode {
    // update 関数内での処理例
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::TreeLoaded(file_node) => {
                self.name = file_node.name;
                self.path = file_node.path;
                self.is_dir = file_node.is_dir;
                self.is_expanded = file_node.is_expanded;
                self.children = file_node.children;

                Task::none()
            }
            Message::ToggleExpand((path, include_file, include_hidden)) => {
                // 再帰的にツリーを更新して is_expanded を切り替える
                self.update_tree_lazy(&path, include_file, include_hidden);
                Task::none()
            }
            Message::SelectFolder(_) => {
                // self.selected_path = Some(path);
                // // ここでファイル一覧の取得などの処理を走らせる
                Task::none()
            }
        }
    }

    fn update_tree_lazy(&mut self, path: &PathBuf, include_file: bool, include_hidden: bool) {
        if self.path == *path {
            self.is_expanded = !self.is_expanded;

            // フォルダが開かれ、かつ中身がまだ空なら読み込む
            if self.is_expanded && self.children.is_empty() {
                if let Ok(entries) = fs::read_dir(&self.path) {
                    for entry in entries.flatten() {
                        if !include_file {
                            if let Ok(file_type) = entry.file_type() {
                                if !file_type.is_dir() {
                                    continue;
                                }
                            }
                        }

                        if !include_hidden {
                            if is_hidden(path) {
                                continue;
                            }
                        }

                        self.children.push(FileNode::new(entry.path(), false)); // 子のさらに下は読み込まない
                    }
                }
            }
            return;
        }

        for child in &mut self.children {
            child.update_tree_lazy(path, include_file, include_hidden);
        }
    }
}

use std::path::Path;

fn is_hidden(path: &Path) -> bool {
    let start_with_dot = path
        .file_name()
        .and_then(|s| s.to_str())
        .map(|s| s.starts_with('.'))
        .unwrap_or(false);

    if start_with_dot {
        return true;
    }

    // 2. Windows固有の属性チェック（必要であれば）
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if let Ok(metadata) = std::fs::metadata(path) {
            if (metadata.file_attributes() & 0x2) != 0 {
                return true;
            }
        }
    }

    false
}
