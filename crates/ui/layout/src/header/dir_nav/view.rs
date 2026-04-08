use iced::Element;
use iced::widget::{button, row, text_input};

use super::DirNav;
use super::message::{Internal, Message};

impl DirNav {
    pub fn view(&self) -> Element<'_, Message> {
        let input = text_input("", &self.processing)
            .on_input(|x| Message::Internal(Internal::Input(x)))
            .on_submit(Message::Internal(Internal::Submit));

        let rfd = button("Folder").on_press(Message::Internal(Internal::RfdOpen));

        row![input, rfd].spacing(10).into()
    }
}
