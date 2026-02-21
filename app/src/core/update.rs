use iced::Task;
use swdir::Swdir;

use super::{App, gallery, message::Message};
use arama_widget::dir_tree;

impl App {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::GalleryMessage(message) => self
                .gallery
                .update(message)
                .map(|message| Message::GalleryMessage(message)),
            Message::DirTreeMessage(message) => {
                let task = self
                    .dir_tree
                    .update(message.clone())
                    .map(|message| Message::DirTreeMessage(message));

                match message {
                    dir_tree::message::Message::DirClick(path) => {
                        // todo dir_node should be got from dir_tree
                        let dir_node = Swdir::default()
                            .set_root_path(path)
                            .set_extension_allowlist(gallery::EXTENSION_ALLOWLIST)
                            .expect("failed to set allowlist")
                            .walk();
                        let _ = self
                            .gallery
                            .update(gallery::message::Message::DirSelect(dir_node));
                        return Task::none();
                    }
                    _ => (),
                }

                task
            }
            Message::ModelLoaderMessage(message) => {
                let task = self.model_loader.update(message);
                task.map(Message::ModelLoaderMessage)
            }
        }
    }
}
