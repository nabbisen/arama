use iced::Task;
use swdir::Swdir;

use super::{App, Dialog, gallery, message::Message};
use arama_widget::{aside, dialog, header};

impl App {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::GalleryMessage(message) => self
                .gallery
                .update(message)
                .map(|message| Message::GalleryMessage(message)),
            Message::HeaderMessage(message) => {
                let output = self.header.update(message);
                match output {
                    header::output::Output::SettingsClick => {
                        self.dialog = Some(Dialog::Settings(dialog::settings::Settings::default()))
                    }
                    _ => (),
                }
                Task::none()
            }
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
            Message::SettingsDialogMessage(_message) => {
                // // Settingsダイアログが開いている時だけupdateを伝播
                // if let Some(Dialog::Settings(settings)) = &mut self.dialog {
                //     // ここでダイアログの `Output`（閉じるとか保存するとか）を受け取って処理することも可能
                //     let _ = settings.update(message);
                // }
                Task::none()
            }
            Message::DialogClose => {
                self.dialog = None;
                Task::none()
            }
        }
    }
}
