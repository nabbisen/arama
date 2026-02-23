use std::path::PathBuf;

use file_node::FileNode;

mod file_node;
pub(super) mod message;
pub mod output;
mod update;
mod view;

#[derive(Clone, Debug)]
pub struct DirTree {
    root: FileNode,
    include_file: bool,
    include_hidden: bool,
    selected_path: Option<PathBuf>,
    processing: bool,
}

impl DirTree {
    pub fn new<T: Into<PathBuf>>(
        path: T,
        include_file: bool,
        include_hidden: bool,
        processing: bool,
    ) -> Self {
        Self {
            root: FileNode::new(path, true, true),
            include_file,
            include_hidden,
            selected_path: None,
            processing,
        }
    }

    pub fn set_processing(&mut self, processing: bool) {
        self.processing = processing;
    }
}
