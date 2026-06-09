use iced::{
    Element,
    widget::{image, mouse_area},
};
use image::Handle;

use super::{ImageCell, message::Message};

impl ImageCell {
    // 'static lifetime against borrow checker in chunk()...map() list operation
    pub fn view(self) -> Element<'static, Message> {
        let handle = Handle::from_path(&self.thumbnail_path);

        let content = mouse_area(
            image(handle)
                .width(self.thumbnail_size)
                .height(self.thumbnail_size)
                .content_fit(iced::ContentFit::Cover),
        )
        .on_enter(Message::ImageCellEnter(self.path.clone()))
        .on_double_click(Message::ImageSelect)
        .on_right_press(Message::ContextMenuOpen)
        .interaction(iced::mouse::Interaction::Pointer);

        content.into()
    }
}
