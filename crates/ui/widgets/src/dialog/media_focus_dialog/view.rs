use iced::widget::{
    image::Handle,
    scrollable::{Direction, Scrollbar},
};
use iced::{
    Element,
    Length::Fill,
    widget::{button, column, container, image, scrollable},
};

use super::{MediaFocusDialog, message::Message};

impl MediaFocusDialog {
    pub fn view(&self) -> Element<'_, Message> {
        let handle = Handle::from_path(&self.path);
        let content = scrollable(
            container(image(handle))
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
        .height(Fill);
        let close_button = container(button("Close").on_press(Message::CloseClick)).center_x(Fill);
        column![content, close_button].into()
    }
}
