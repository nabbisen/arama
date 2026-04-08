pub(super) mod message;
pub(super) mod update;
pub(super) mod view;

#[derive(Clone, Debug)]
pub struct DirNav {
    original_path_str: String,
    input_str: String,
}

impl DirNav {
    pub fn new(path_str: &str) -> Self {
        Self {
            original_path_str: path_str.to_owned(),
            input_str: path_str.to_owned(),
        }
    }

    pub fn update_path(&mut self, path_str: &str) {
        self.original_path_str = path_str.to_owned();
        self.input_str = path_str.to_owned();
    }
}
