use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
pub struct Settings {
    pub root_dir_path: String,
}
