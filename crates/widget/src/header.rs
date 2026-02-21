mod dir_nav;
pub mod message;
mod settings;
mod update;
pub mod view;

use dir_nav::DirNav;
use settings::Settings;

#[derive(Clone, Debug, Default)]
pub struct Header {
    dir_nav: DirNav,
    settings: Settings,
}
