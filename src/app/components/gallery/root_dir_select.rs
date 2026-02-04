use std::path::PathBuf;

pub mod message;
mod update;
mod view;

#[derive(Default)]
pub struct RootDirSelect {
    selected_path: Option<PathBuf>,
}
