use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

use file_node::FileNode;

mod file_node;
pub(super) mod message;
pub mod output;
mod update;
mod view;

const DOUBLE_CLICK_INTERVAL_MILLIS: Duration = Duration::from_millis(600);

#[derive(Clone, Debug)]
pub struct DirTree {
    root: FileNode,
    include_file: bool,
    include_hidden: bool,
    dir_last_clicked: Option<(PathBuf, Instant)>,
    selected_path: Option<PathBuf>,
}

impl DirTree {
    pub fn new<T: Into<PathBuf>>(path: T, include_file: bool, include_hidden: bool) -> Self {
        Self {
            root: FileNode::new(path, true, true),
            include_file,
            include_hidden,
            dir_last_clicked: None,
            selected_path: None,
        }
    }
}
