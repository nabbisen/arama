pub mod message;
mod update;
pub mod view;

#[derive(Clone, Debug)]
pub struct Footer {
    files_count: usize,
    dirs_count: usize,
}

impl Footer {
    pub fn new(files_count: usize, dirs_count: usize) -> Self {
        Self {
            files_count,
            dirs_count,
        }
    }

    pub fn update_count(&mut self, files_count: usize, dirs_count: usize) {
        self.files_count = files_count;
        self.dirs_count = dirs_count;
    }
}
