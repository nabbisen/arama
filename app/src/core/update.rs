use arama_env::{IMAGE_EXTENSION_ALLOWLIST, VIDEO_EXTENSION_ALLOWLIST};
use arama_ui_main::{components::gallery::gallery_settings, views::gallery};
use iced::Task;
use swdir::Swdir;

use super::{App, Dialog, message::Message};
use arama_ui_layout::{aside, header};
use arama_ui_widgets::dialog::{media_focus_dialog, settings_dialog, similar_pairs_dialog};

impl App {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::GalleryMessage(message) => {
                let task = self
                    .gallery
                    .update(message.clone())
                    .map(|message| Message::GalleryMessage(message));
                match message {
                    gallery::message::Message::ImageCached(_) => {
                        self.processing = false;
                        self.aside.set_processing(self.processing);
                    }
                    gallery::message::Message::ImageSelect(path) => {
                        self.dialog = Some(Dialog::MediaFocusDialog(
                            media_focus_dialog::MediaFocusDialog::new(path),
                        ));
                    }
                    gallery::message::Message::SimilarPairsOpen => {
                        // todo: error handling
                        let dialog = similar_pairs_dialog::SimilarPairsDialog::new(
                            self.gallery.dir_node().unwrap(),
                            None,
                            self.gallery.thumbnail_size(),
                        );
                        self.dialog = Some(Dialog::SimilarPairsDialog(dialog.clone()));
                        return dialog
                            .default_task()
                            .map(Message::SimilarPairsDialogMessage);
                    }
                    gallery::message::Message::GallerySettingsMessage(message) => {
                        let output = self.gallery.gallery_settings.update(message);
                        match output {
                            Some(gallery_settings::output::Output::TargetMediaTypeChange(
                                media_type,
                            )) => {
                                self.target_media_type = media_type;
                            }
                            _ => (),
                        }
                    }
                    _ => (),
                }
                task
            }
            Message::HeaderMessage(message) => {
                let output = self.header.update(message);
                match output {
                    header::output::Output::SettingsClick => {
                        self.dialog = Some(Dialog::SettingsDialog(
                            settings_dialog::SettingsDialog::default(),
                        ))
                    }
                    _ => (),
                }
                Task::none()
            }
            Message::AsideMessage(message) => {
                let output = self.aside.update(message.clone());

                match output {
                    Some(aside::output::Output::DirClick(path)) => {
                        self.processing = true;

                        // todo dir_node should be got from dir_tree
                        let mut extension_allowlist: Vec<&str> = vec![];
                        if self.target_media_type.include_image {
                            extension_allowlist.extend(IMAGE_EXTENSION_ALLOWLIST);
                        }
                        if self.target_media_type.include_video {
                            extension_allowlist.extend(VIDEO_EXTENSION_ALLOWLIST);
                        }

                        let dir_node = Swdir::default()
                            .set_root_path(path)
                            .set_extension_allowlist(&extension_allowlist)
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
            Message::MediaFocusDialogMessage(message) => {
                if let Some(Dialog::MediaFocusDialog(x)) = &mut self.dialog {
                    // ここでダイアログの `Output`（閉じるとか保存するとか）を受け取って処理することも可能
                    let output = x.update(message);
                    match output {
                        Some(media_focus_dialog::output::Output::CloseClick) => {
                            self.dialog = None;
                        }
                        _ => (),
                    }
                }
                Task::none()
            }
            Message::SimilarPairsDialogMessage(message) => {
                let _ = match &self.dialog {
                    Some(Dialog::SimilarPairsDialog(dialog)) => {
                        let output = dialog.to_owned().update(message);
                        match output {
                            Some(similar_pairs_dialog::output::Output::EmbeddingsReady(pairs)) => {
                                self.dialog = Some(Dialog::SimilarPairsDialog(
                                    similar_pairs_dialog::SimilarPairsDialog::new(
                                        self.gallery.dir_node().unwrap(),
                                        Some(pairs),
                                        self.gallery.thumbnail_size(),
                                    ),
                                ));
                            }
                            None => (),
                        }
                    }
                    _ => (),
                };
                Task::none()
            }
            Message::SettingsDialogMessage(message) => {
                // Settingsダイアログが開いている時だけupdateを伝播
                if let Some(Dialog::SettingsDialog(settings)) = &mut self.dialog {
                    // ここでダイアログの `Output`（閉じるとか保存するとか）を受け取って処理することも可能
                    let task = settings.update(message.clone());
                    return task.map(Message::SettingsDialogMessage);
                }
                Task::none()
            }
            Message::DialogClose => {
                self.dialog = None;
                Task::none()
            }
        }
    }
}
