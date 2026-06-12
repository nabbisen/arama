use std::path::PathBuf;

use iced_swdir_tree::{DirectoryFilter, DirectoryTree};

pub mod message;
mod update;
pub mod view;

pub struct Aside {
    pub(crate) tree: DirectoryTree,
    pub(crate) processing: bool,
}

impl Aside {
    pub fn new(path: PathBuf, processing: bool) -> Self {
        let tree = DirectoryTree::new(path).with_filter(DirectoryFilter::FoldersOnly);
        Self { tree, processing }
    }

    /// Rebuild the tree rooted at `path`.
    ///
    /// Called when the user navigates to a new directory via the header
    /// input or file-picker. `DirectoryTree::new` is lazy — no I/O
    /// occurs at construction; the first expansion triggers an async scan.
    pub fn update_dir_tree(&mut self, path: &PathBuf) {
        self.tree = DirectoryTree::new(path.clone()).with_filter(DirectoryFilter::FoldersOnly);
    }

    pub fn set_processing(&mut self, processing: bool) {
        self.processing = processing;
    }
}
