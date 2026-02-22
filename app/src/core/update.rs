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
                let output = self.aside.update(message.clone());

                match output {
                    Some(aside::output::Output::DirClick(path)) => {
                        // todo dir_node should be got from dir_tree
                        let dir_node = Swdir::default()
                            .set_root_path(path)
                            .set_extension_allowlist(gallery::EXTENSION_ALLOWLIST)
                            .expect("failed to set allowlist")
                            .walk();
                        let task = self
                            .gallery
                            .update(gallery::message::Message::DirSelect(dir_node))
                            .map(Message::GalleryMessage);
                        return task;
                    }
                    _ => (),
                }
                Task::none()
            }
            Message::FooterMessage(message) => self
                .footer
                .update(message)
                .map(|message| Message::FooterMessage(message)),
        }
    }
}
