use std::path::PathBuf;

use arama_ui_widgets::dir_tree::DirTree;

pub mod message;
mod update;
pub mod view;

#[derive(Clone, Debug)]
pub struct Aside {
    dir_tree: DirTree,
}

impl Aside {
    pub fn new<T: Into<PathBuf> + Clone>(
        path: T,
        include_file: bool,
        include_hidden: bool,
        processing: bool,
    ) -> Self {
        let dir_tree = DirTree::new(path, include_file, include_hidden, processing);
        Self { dir_tree }
    }

    pub fn set_processing(&mut self, processing: bool) {
        self.dir_tree.set_processing(processing);
    }
}
