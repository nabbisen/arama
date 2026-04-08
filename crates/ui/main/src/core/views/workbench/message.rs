use crate::components::workbench::image_cell;

#[derive(Debug, Clone)]
pub enum Message {
    ImageCellMessage(image_cell::message::Message),
    CursorExit,
}
