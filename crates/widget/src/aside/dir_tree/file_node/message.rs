use std::path::PathBuf;

use super::FileNode;

#[derive(Debug, Clone)]
pub enum Message {
    TreeLoaded(FileNode),
    ToggleExpand((PathBuf, bool, bool)),
    DirClick(PathBuf),
}
