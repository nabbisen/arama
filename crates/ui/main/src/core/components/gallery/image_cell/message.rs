use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum Message {
    ImageCellEnter(PathBuf),
    ImageSelect,
    ContextMenuOpen,
}
