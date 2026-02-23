use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum Output {
    DirSelect(PathBuf),
}
