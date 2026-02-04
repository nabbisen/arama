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

                self.image_similarity_update()
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
            Message::ImageSelect(path) => {
                self.selected_source_image = Some(path);

                self.image_similarity_update()
            }
            Message::ImageSimilarityCompleted(image_paths) => {
                self.image_paths = image_paths;
                Task::none()
            }
        }
    }

    fn image_similarity_update(&mut self) -> Task<Message> {
        let selected_source_image = match self.selected_source_image.as_ref() {
            Some(x) => x.clone(),
            None => return Task::none(),
        };

        self.image_paths = self
            .image_paths
            .clone()
            .into_iter()
            .map(|x| (x.0, None))
            .collect();

        let image_paths = self.image_paths.clone();
        Task::perform(
            async move {
                let image_tensor = ImageTensor::new(
                    selected_source_image.as_path(),
                    image_paths.iter().map(|x| x.0.as_path()).collect(),
                );
                // println!("{:?}", image_tensor);
                if let Ok(image_tensor) = image_tensor {
                    if let Ok(targets) = image_tensor.targets {
                        return targets.into_iter().map(|x| (x.0, Some(x.1))).collect();
                    }
                }
                image_paths
            },
            Message::ImageSimilarityCompleted,
        )
    }
}
