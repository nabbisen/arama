use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum Output {
    DirClick(PathBuf),
}
