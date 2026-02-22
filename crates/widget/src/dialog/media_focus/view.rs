use iced::Element;
use iced::Length::Fill;
use iced::widget::image::Handle;
use iced::widget::{button, column, container, image, scrollable};

use super::{MediaFocus, message::Message};

impl MediaFocus {
    pub fn view(&self) -> Element<'_, Message> {
        let handle = Handle::from_path(&self.path);
        // todo: improve view
        let content = scrollable(
            container(image(handle))
                .width(Fill)
                .height(Fill)
                .center(Fill)
                .padding(40),
        );
        let close_button = container(button("Close").on_press(Message::CloseClick)).center_x(Fill);
        column![content, close_button].into()
    }
}
