use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum Message {
    OpenWithDefault(PathBuf),
    FileManagerShow(PathBuf),
}
