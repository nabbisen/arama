use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum Message {
    ImageSelect(PathBuf),
    ContextMenuOpen(PathBuf),
}
