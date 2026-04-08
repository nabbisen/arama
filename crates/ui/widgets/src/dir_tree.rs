use std::path::PathBuf;

use file_node::FileNode;

mod file_node;
pub mod message;
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
    pub fn new<T: Into<PathBuf> + Clone>(
        path: T,
        include_file: bool,
        include_hidden: bool,
        processing: bool,
    ) -> Self {
        let selected_path = Some(path.clone().into());

        Self {
            root: FileNode::new(path, true, true),
            include_file,
            include_hidden,
            selected_path,
            processing,
        }
    }

    pub fn update_selected_path<T: Into<PathBuf> + Clone>(&mut self, path: T) {
        self.root = FileNode::new(path.clone(), true, true);
        self.selected_path = Some(path.into());
    }

    pub fn set_processing(&mut self, processing: bool) {
        self.processing = processing;
    }
}
