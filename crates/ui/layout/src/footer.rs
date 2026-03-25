pub mod message;
pub mod thumbnail_size_slider;
mod update;
pub mod view;

use std::path::PathBuf;

use thumbnail_size_slider::ThumbnailSizeSlider;

#[derive(Clone, Debug)]
pub struct Footer {
    thumbnail_size_slider: ThumbnailSizeSlider,
    files_count: usize,
    dirs_count: usize,
    image_cell_path: Option<PathBuf>,
}

impl Footer {
    pub fn new(thumbnail_size: u16, files_count: usize, dirs_count: usize) -> Self {
        Self {
            thumbnail_size_slider: ThumbnailSizeSlider::new(thumbnail_size),
            files_count,
            dirs_count,
            image_cell_path: None,
        }
    }

    pub fn thumbnail_size(&self) -> u16 {
        self.thumbnail_size_slider.value
    }

    pub fn update_count(&mut self, files_count: usize, dirs_count: usize) {
        self.files_count = files_count;
        self.dirs_count = dirs_count;
    }

    pub fn update_image_cell_path(&mut self, path: Option<PathBuf>) {
        self.image_cell_path = path;
    }
}
