mod dir_nav;
pub mod message;
mod settings_nav;
mod update;
pub mod view;

use dir_nav::DirNav;
use settings_nav::SettingsNav;

#[derive(Clone, Debug, Default)]
pub struct Header {
    dir_nav: DirNav,
    settings_nav: SettingsNav,
    embedding_cached: bool,
}

impl Header {
    pub fn set_embedding_cached(&mut self, embedding_cached: bool) {
        self.embedding_cached = embedding_cached;
    }
}
