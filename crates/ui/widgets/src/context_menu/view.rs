use arama_i18n::t;
use iced::{
    Element,
    widget::{button, column, container, row, space, text},
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
                        button(text(t("context_menu.open_with_default")))
                            .on_press(Message::OpenWithDefault(path.to_path_buf())),
                        button(text(t("context_menu.file_manager")))
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
