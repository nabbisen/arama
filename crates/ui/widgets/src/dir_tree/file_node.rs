use std::fs;
use std::path::{Path, PathBuf};

use crate::dir_tree::file_node::util::is_hidden;

pub(super) mod message;
mod update;
mod util;
mod view;

#[derive(Debug, Clone)]
pub struct FileNode {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub is_expanded: bool,
    pub children: Vec<FileNode>, // フォルダの場合の子要素
}

impl FileNode {
    /// 指定したパスからノードを作成（再帰的に読み込む場合は recursive = true）
    pub fn new<T: Into<PathBuf>>(path: T, recursive: bool, init: bool) -> Self {
        let path = path
            .into()
            .canonicalize()
            .expect("failed to canonicalize path");

        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("/")
            .to_string();

        let is_dir = path.is_dir();

        let children = if !init && is_dir && recursive {
            FileNode::children(&path, recursive, init)
        } else {
            vec![]
        };

        let base = Self {
            name,
            path,
            is_dir,
            is_expanded: false,
            children,
        };

        if init {
            parent_file_node(&base, 0)
        } else {
            base
        }
    }

    pub fn children(path: &Path, recursive: bool, init: bool) -> Vec<FileNode> {
        let mut children = vec![];

        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries
                .flatten()
                .filter(|x| x.path().is_dir() && !is_hidden(x.path().as_ref()))
            {
                children.push(FileNode::new(entry.path(), recursive, init));
            }
        }

        // 名前順にソート（ディレクトリを優先）
        children.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then(a.name.cmp(&b.name)));

        children
    }
}

fn parent_file_node(file_node: &FileNode, step: usize) -> FileNode {
    let path = if let Some(path) = file_node.path.parent() {
        path
    } else {
        return file_node.to_owned();
    };

    let children = if step == 0 {
        FileNode::children(path, false, false)
    } else {
        vec![file_node.to_owned()]
    };

    let base = FileNode {
        name: path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("/")
            .to_string(),
        path: path.to_path_buf(),
        is_dir: path.is_dir(),
        is_expanded: true,
        children,
    };

    parent_file_node(&base, step + 1)
}
