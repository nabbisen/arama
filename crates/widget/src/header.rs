mod dir_nav;
pub mod message;
pub mod output;
mod settings_nav;
mod update;
pub mod view;

use dir_nav::DirNav;
use settings_nav::SettingsNav;

#[derive(Clone, Debug, Default)]
pub struct Header {
    dir_nav: DirNav,
    settings_nav: SettingsNav,
}
