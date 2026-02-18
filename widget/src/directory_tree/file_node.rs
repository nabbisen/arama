use std::fs;
use std::path::PathBuf;

pub(super) mod message;
pub(super) mod update;
pub(super) mod view;

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
    pub fn new<T: Into<PathBuf>>(path: T, recursive: bool) -> Self {
        let path = path.into();

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
                    children.push(FileNode::new(entry.path(), true));
                }
            }
        }

        // 名前順にソート（ディレクトリを優先）
        children.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then(a.name.cmp(&b.name)));

        Self {
            name,
            path,
            is_dir,
            is_expanded: false,
            children,
        }
    }
}
