use iced::{
    Element,
    widget::{button, column, container, row, space},
};

use super::{ContextMenu, ContextMenuState, message::Message};

impl ContextMenu {
    pub fn view(&self) -> Element<'_, Message> {
        let ret = match &self.state {
            ContextMenuState::ImageCell(path) => container(column![
                space().height(self.point.y),
                row![
                    space().width(self.point.x),
                    column![
                        button("file manager")
                            .on_press(Message::FileManagerShow(path.to_path_buf())),
                    ]
                    .width(self.thumbnail_size as f32)
                    .spacing(5)
                ]
            ]),
            ContextMenuState::None => container(space()),
        };
        ret.into()
    }
}
