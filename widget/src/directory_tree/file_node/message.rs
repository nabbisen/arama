use std::path::PathBuf;

use super::FileNode;

#[derive(Debug, Clone)]
pub enum Message {
    TreeLoaded(FileNode),
    ToggleExpand((PathBuf, bool, bool)), // フォルダの開閉
    DirectoryClick(PathBuf),             // フォルダの選択
}
