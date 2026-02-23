use iced::widget::{
    image::Handle,
    row,
    scrollable::{Direction, Scrollbar},
    text, toggler,
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
        let header = container(row![text("Actual size"), view_size_toggler].spacing(10))
            .center_x(Fill)
            .padding(10);

        let close_button = button("Close").on_press(Message::CloseClick);
        let footer = container(close_button).center_x(Fill).padding(10);

        column![header, content, footer].into()
    }
}
