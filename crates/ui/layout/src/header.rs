mod dir_nav;
pub mod message;
mod settings_nav;
mod update;
pub mod view;

use dir_nav::DirNav;
use settings_nav::SettingsNav;

#[derive(Clone, Debug)]
pub struct Header {
    dir_nav: DirNav,
    settings_nav: SettingsNav,
    embedding_cached: bool,
}

impl Header {
    pub fn new(path: &str) -> Self {
        Self {
            dir_nav: DirNav::new(path),
            settings_nav: SettingsNav::default(),
            embedding_cached: false,
        }
    }

    pub fn set_embedding_cached(&mut self, embedding_cached: bool) {
        self.embedding_cached = embedding_cached;
    }
}
