use iced::Task;
use swdir::DirNode;

use crate::app::{
    components::gallery::{menus, root_dir_select},
    utils::gallery::image_similarity::ImageSimilarity,
};

use super::{Gallery, message::Message};

impl Gallery {
    // アプリケーション初期化時に画像を読み込むTaskを発行
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ImagesLoaded(dir_node) => {
                self.dir_node = dir_node;

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
                            self.dir_node = DirNode::with_path(path);
                            return Task::perform(
                                super::util::load_images(self.dir_node.path.clone()),
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
            Message::ImageSimilarityCompleted(image_similarity) => {
                self.image_similarity = image_similarity;
                Task::none()
            }
        }
    }

    fn image_similarity_update(&mut self) -> Task<Message> {
        let selected_source_image = match self.selected_source_image.as_ref() {
            Some(x) => x.clone(),
            None => return Task::none(),
        };

        self.image_similarity = ImageSimilarity::default();

        let dir_node = self.dir_node.clone();
        Task::perform(
            async move {
                let image_similarity =
                    ImageSimilarity::calculate(selected_source_image.as_path(), &dir_node);
                // println!("{:?}", image_tensor);
                match image_similarity {
                    Ok(image_similarity) => image_similarity,
                    Err(_) => ImageSimilarity::default(), // todo: error handling
                }
            },
            Message::ImageSimilarityCompleted,
        )
    }
}
