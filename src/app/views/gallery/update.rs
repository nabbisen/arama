use iced::Task;

use crate::app::{
    components::gallery::{menus, root_dir_select},
    image_tensor::ImageTensor,
};

use super::{Gallery, message::Message};

impl Gallery {
    // アプリケーション初期化時に画像を読み込むTaskを発行
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ImagesLoaded(paths) => {
                self.image_paths = paths.into_iter().map(|x| (x, None)).collect();

                if 0 < self.image_paths.len() {
                    let image_tensor = ImageTensor::new(
                        self.image_paths[0].0.as_path(),
                        self.image_paths.iter().map(|x| x.0.as_path()).collect(),
                    );
                    // println!("{:?}", image_tensor);
                    if let Ok(image_tensor) = image_tensor {
                        if let Ok(targets) = image_tensor.targets {
                            self.image_paths =
                                targets.into_iter().map(|x| (x.0, Some(x.1))).collect();
                        }
                    }
                }

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
