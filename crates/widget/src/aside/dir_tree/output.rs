use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum Output {
    DirClick(PathBuf),
    // todo: double click event ?
    #[allow(dead_code)]
    DirDoubleClick(PathBuf),
}
