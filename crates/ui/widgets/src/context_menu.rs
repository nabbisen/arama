use std::path::PathBuf;

use iced::Point;

pub mod message;
mod update;
mod view;

#[derive(Clone, Debug)]
pub enum ContextMenuState {
    ImageCell(PathBuf),
    None,
}

#[derive(Clone, Debug)]
pub struct ContextMenu {
    pub state: ContextMenuState,
    point: Point,
    thumbnail_size: u16,
}

impl ContextMenu {
    pub fn new(point: Point, thumbnail_size: u16) -> Self {
        Self {
            state: ContextMenuState::None,
            point,
            thumbnail_size,
        }
    }

    pub fn update_point(&mut self, point: Point) {
        self.point = point;
    }
}
