mod dir_tree;
pub mod message;
mod update;
pub mod view;

use std::path::PathBuf;

use dir_tree::DirTree;

#[derive(Clone, Debug)]
pub struct Aside {
    dir_tree: DirTree,
}

impl Aside {
    pub fn new<T: Into<PathBuf>>(path: T, include_file: bool, include_hidden: bool) -> Self {
        let dir_tree = DirTree::new(path, include_file, include_hidden);
        Self { dir_tree }
    }
}
