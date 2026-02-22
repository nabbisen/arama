use std::path::PathBuf;

pub mod message;
pub mod output;
mod update;
mod view;

#[derive(Clone, Debug)]
pub struct MediaFocus {
    path: PathBuf,
}

impl MediaFocus {
    pub fn new<T: Into<PathBuf>>(path: T) -> Self {
        Self { path: path.into() }
    }
}
