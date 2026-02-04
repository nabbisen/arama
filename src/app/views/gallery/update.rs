use iced::Task;

use crate::app::components::gallery::{menus, root_dir_select};

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
            Message::RootDirSelectMessage(message) => {
                let task = self
                    .root_dir_select
                    .update(message.clone())
                    .map(|message| Message::RootDirSelectMessage(message));

                match message {
                    root_dir_select::message::Message::DialogClose(path) => {
                        if let Some(path) = path {
                            self.root_dir = path;
                            return Task::perform(
                                super::util::load_images(self.root_dir.clone()),
                                super::message::Message::ImagesLoaded,
                            );
                        }
                    }
                    _ => (),
                }

                task
            }
        }
    }
}
