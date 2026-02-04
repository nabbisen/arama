use iced::Task;

use crate::app::components::gallery::menus;

use super::{Gallery, message::Message};

impl Gallery {
    // アプリケーション初期化時に画像を読み込むTaskを発行
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ImagesLoaded(paths) => {
                self.image_paths = paths;
                Task::none()
            }
            Message::MenusMessage(message) => match message {
                menus::message::Message::ScaleUp => {
                    if self.thumbnail_size <= 600 {
                        self.thumbnail_size += 20;
                    }
                    Task::none()
                }
                menus::message::Message::ScaleDown => {
                    if 40 <= self.thumbnail_size {
                        self.thumbnail_size -= 20;
                    }
                    Task::none()
                }
                menus::message::Message::Quit => iced::exit(),
            },
        }
    }
}
