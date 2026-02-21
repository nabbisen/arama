use iced::Task;
use swdir::Swdir;

use super::{App, gallery, message::Message};
use arama_widget::aside;

impl App {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::GalleryMessage(message) => self
                .gallery
                .update(message)
                .map(|message| Message::GalleryMessage(message)),
            Message::HeaderMessage(message) => self
                .header
                .update(message)
                .map(|message| Message::HeaderMessage(message)),
            Message::AsideMessage(message) => {
                let task = self
                    .aside
                    .update(message.clone())
                    .map(|message| Message::AsideMessage(message));

                match message {
                    aside::message::Message::DirClick(path) => {
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
            Message::FooterMessage(message) => self
                .footer
                .update(message)
                .map(|message| Message::FooterMessage(message)),
        }
    }
}
