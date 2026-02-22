use std::fs;
use std::path::PathBuf;

pub(super) mod message;
mod output;
mod update;
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
        let mut children = Vec::new();

        if is_dir && recursive {
            if let Ok(entries) = fs::read_dir(&path) {
                for entry in entries.flatten() {
                    // ここで再帰呼び出し
                    children.push(FileNode::new(entry.path(), true, false));
                }
            }
        }

        // 名前順にソート（ディレクトリを優先）
        children.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then(a.name.cmp(&b.name)));

        let base = Self {
            name,
            path,
            is_dir,
            is_expanded: false,
            children,
        };

        if init { parent_file_node(&base) } else { base }
    }
}

fn parent_file_node(file_node: &FileNode) -> FileNode {
    let path = if let Some(path) = file_node.path.parent() {
        path
    } else {
        return file_node.to_owned();
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
        children: vec![file_node.to_owned()],
    };

    parent_file_node(&base)
}
