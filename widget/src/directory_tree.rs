use std::path::PathBuf;

use crate::directory_tree::file_node::FileNode;

mod file_node;
pub mod message;
pub mod update;
pub mod view;

#[derive(Clone, Debug)]
pub struct DirectoryTree {
    root: FileNode,
    include_file: bool,
    include_hidden: bool,
    selected_path: Option<PathBuf>,
}

impl DirectoryTree {
    /// 指定したパスからノードを作成（再帰的に読み込む場合は recursive = true）
    pub fn new<T: Into<PathBuf>>(path: T, include_file: bool, include_hidden: bool) -> Self {
        Self {
            root: FileNode::new(path, true, true),
            include_file,
            include_hidden,
            selected_path: None,
        }
    }
}
