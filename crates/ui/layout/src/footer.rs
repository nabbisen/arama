pub mod message;
pub mod thumbnail_size_slider;
mod update;
pub mod view;

use thumbnail_size_slider::ThumbnailSizeSlider;

#[derive(Clone, Debug)]
pub struct Footer {
    thumbnail_size_slider: ThumbnailSizeSlider,
    files_count: usize,
    dirs_count: usize,
}

impl Footer {
    pub fn new(files_count: usize, dirs_count: usize) -> Self {
        Self {
            thumbnail_size_slider: ThumbnailSizeSlider::default(),
            files_count,
            dirs_count,
        }
    }

    pub fn thumbnail_size(&self) -> u16 {
        self.thumbnail_size_slider.value
    }

    pub fn update_count(&mut self, files_count: usize, dirs_count: usize) {
        self.files_count = files_count;
        self.dirs_count = dirs_count;
    }
}
