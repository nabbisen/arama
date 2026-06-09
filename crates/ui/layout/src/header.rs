mod dir_nav;
pub mod message;
mod update;
pub mod view;

use dir_nav::DirNav;

#[derive(Clone, Debug)]
pub struct Header {
    dir_nav: DirNav,
    embedding_cached: bool,
}

impl Header {
    pub fn new(path: &str) -> Self {
        Self {
            dir_nav: DirNav::new(path),
            embedding_cached: false,
        }
    }

    pub fn set_embedding_cached(&mut self, embedding_cached: bool) {
        self.embedding_cached = embedding_cached;
    }
}
