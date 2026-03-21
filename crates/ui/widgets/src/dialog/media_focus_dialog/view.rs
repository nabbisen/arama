use iced::widget::{
    Row,
    image::Handle,
    mouse_area, row,
    scrollable::{Direction, Scrollbar},
    space, text, toggler,
};
use iced::{
    Element,
    Length::Fill,
    widget::{button, column, container, image, scrollable},
};

use super::{MediaFocusDialog, message::Message};

impl MediaFocusDialog {
    pub fn view(&self) -> Element<'_, Message> {
        let path_text = text(self.path.to_string_lossy().to_string());
        let header = container(path_text).center_x(Fill);

        let handle = Handle::from_path(&self.path);
        let img = image(handle);
        let content = if self.actual_size {
            scrollable(
                container(img)
                    .width(Fill)
                    .height(Fill)
                    .center(Fill)
                    .padding(10),
            )
            .direction(Direction::Both {
                vertical: Scrollbar::default(),
                horizontal: Scrollbar::default(),
            })
            .width(Fill)
            .height(Fill)
        } else {
            scrollable(container(img).width(Fill).center(Fill))
                .width(Fill)
                .height(Fill)
        };

        let view_size_toggler = toggler(self.actual_size).on_toggle(Message::ViewSizeToggle);
        let view_control =
            container(row![text("Actual size"), view_size_toggler].spacing(10)).center_x(Fill);

        // todo
        let similar_media_items = self.similar_media.iter().fold(row![], |r: Row<_>, x| {
            let handle = Handle::from_path(if let Some(thumbnail_path) = &x.thumbnail_path {
                thumbnail_path.to_owned()
            } else {
                x.path.to_owned()
            });
            let item = mouse_area(image(handle)).on_enter(Message::MediaItemEnter(x.path.clone()));
            r.push(column![item, text(x.similarity)].spacing(5))
        });
        let similar_media_content =
            if let Some(hovered_media_item_path) = &self.hovered_media_item_path {
                column![
                    similar_media_items,
                    text(hovered_media_item_path).height(20)
                ]
            } else {
                column![similar_media_items, space().height(20)]
            };
        let similar_media = mouse_area(scrollable(container(similar_media_content)))
            .on_exit(Message::MediaItemExit);

        let close_button = button("Close").on_press(Message::CloseClick);
        let footer = container(close_button).center_x(Fill).padding(10);

        column![header, content, view_control, similar_media, footer]
            .spacing(10)
            .into()
    }
}
