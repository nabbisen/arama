pub(super) mod message;
pub(super) mod update;
pub(super) mod view;

#[derive(Clone, Debug)]
pub struct DirNav {
    path: String,
    processing: String,
}

impl DirNav {
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_owned(),
            processing: path.to_owned(),
        }
    }
}
