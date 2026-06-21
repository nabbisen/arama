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

    /// Sync the displayed path after an external navigation (e.g. aside tree click).
    pub(super) fn set_path(&mut self, path: &str) {
        self.path = path.to_owned();
        self.processing = path.to_owned();
    }
}
