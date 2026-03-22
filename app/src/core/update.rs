use arama_env::{IMAGE_EXTENSION_ALLOWLIST, VIDEO_EXTENSION_ALLOWLIST};
use arama_ui_main::{
    components::gallery::{gallery_settings, image_cell},
    views::gallery,
};
use iced::Task;
use swdir::{Recurse, Swdir};

use super::{App, ContextMenu, Dialog, message::Message};
use arama_ui_layout::{aside, header};
use arama_ui_widgets::{
    dialog::{media_focus_dialog, settings_dialog, similar_pairs_dialog},
    dir_tree,
};

impl App {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SetupMessage(message) => {
                let task = self
                    .setup
                    .update(message.clone())
                    .map(Message::SetupMessage);
                if self.setup.finished {
                    self.gallery
                        .default_task()
                        .map(|message| Message::GalleryMessage(message))
                } else {
                    task
                }
            }
            Message::GalleryMessage(message) => {
                let task = self
                    .gallery
                    .update(message.clone())
                    .map(Message::GalleryMessage);
                match message {
                    gallery::message::Message::ImageCached(_) => {
                        self.processing = false;
                        self.aside.set_processing(self.processing);
                    }
                    gallery::message::Message::SimilarPairsOpen => {
                        // todo: error handling
                        let dialog = similar_pairs_dialog::SimilarPairsDialog::new(
                            self.gallery.dir_node().unwrap(),
                            None,
                        );
                        self.dialog = Some(Dialog::SimilarPairsDialog(dialog.clone()));
                        return dialog
                            .default_task()
                            .map(Message::SimilarPairsDialogMessage);
                    }
                    gallery::message::Message::GallerySettingsMessage(message) => {
                        let _ = self.gallery.gallery_settings.update(message.clone());
                        match message {
                            gallery_settings::message::Message::TargetMediaTypeChanged(
                                target_media_type,
                            ) => {
                                self.settings.target_media_type = target_media_type;
                                self.save_settings();
                            }
                            gallery_settings::message::Message::SubDirDepthLimitChanged(value) => {
                                self.settings.sub_dir_depth_limit = value;
                                self.save_settings();
                            }
                            _ => (),
                        }
                    }
                    gallery::message::Message::ImageCellMessage(message) => match message {
                        image_cell::message::Message::ImageSelect(path) => {
                            let media_focus_dialog =
                                media_focus_dialog::MediaFocusDialog::new(path);
                            let dialog = Dialog::MediaFocusDialog(media_focus_dialog.clone());
                            let default_task = media_focus_dialog.default_task();

                            self.dialog = Some(dialog);

                            return Task::batch([
                                task,
                                default_task.map(Message::MediaFocusDialogMessage),
                            ]);
                        }
                        image_cell::message::Message::ContextMenuOpen(path) => {
                            match self.context_menu {
                                ContextMenu::None => {
                                    self.context_menu = ContextMenu::ImageCell(path)
                                }
                                _ => self.context_menu = ContextMenu::None,
                            }
                        }
                    },
                    gallery::message::Message::DirSelect(_) => (),
                    gallery::message::Message::EmbeddingCached(_) => (),
                }
                task
            }
            Message::HeaderMessage(message) => {
                let task = self
                    .header
                    .update(message.clone())
                    .map(Message::HeaderMessage);
                match message {
                    header::message::Message::SettingsClick => {
                        self.dialog = Some(Dialog::SettingsDialog(
                            settings_dialog::SettingsDialog::default(),
                        ))
                    }
                    _ => (),
                }
                task
            }
            Message::AsideMessage(message) => {
                let task = self.aside.update(message.clone());

                match message {
                    aside::message::Message::DirTreeMessage(message) => {
                        match message {
                            dir_tree::message::Message::DirClick(path) => {
                                self.processing = true;

                                self.settings.root_dir_path = path.to_string_lossy().to_string();
                                self.save_settings();

                                // todo dir_node should be got from dir_tree
                                let mut extension_allowlist: Vec<&str> = vec![];
                                if self.settings.target_media_type.include_image {
                                    extension_allowlist.extend(IMAGE_EXTENSION_ALLOWLIST);
                                }
                                if self.settings.target_media_type.include_video {
                                    extension_allowlist.extend(VIDEO_EXTENSION_ALLOWLIST);
                                }

                                let recurse = if 0 < self.settings.sub_dir_depth_limit {
                                    Recurse {
                                        enabled: true,
                                        depth_limit: Some(self.settings.sub_dir_depth_limit.into()),
                                    }
                                } else {
                                    Recurse {
                                        enabled: false,
                                        depth_limit: None,
                                    }
                                };

                                let dir_node = Swdir::default()
                                    .set_root_path(path)
                                    .set_extension_allowlist(&extension_allowlist)
                                    .expect("failed to set allowlist")
                                    .set_recurse(recurse)
                                    .walk();

                                let task = self
                                    .gallery
                                    .update(gallery::message::Message::DirSelect(dir_node))
                                    .map(Message::GalleryMessage);
                                return task;
                            }
                            _ => (),
                        }
                    }
                }

                task.map(Message::AsideMessage)
            }
            Message::FooterMessage(message) => self
                .footer
                .update(message)
                .map(|message| Message::FooterMessage(message)),
            Message::MediaFocusDialogMessage(message) => {
                if let Some(Dialog::MediaFocusDialog(x)) = &mut self.dialog {
                    let task = x.update(message.clone());
                    match message {
                        media_focus_dialog::message::Message::CloseClick => self.dialog = None,
                        _ => (),
                    }
                    return task.map(Message::MediaFocusDialogMessage);
                }
                Task::none()
            }
            Message::SimilarPairsDialogMessage(message) => {
                if let Some(Dialog::SimilarPairsDialog(x)) = &mut self.dialog {
                    let task = x
                        .update(message.clone())
                        .map(Message::SimilarPairsDialogMessage);
                    match message {
                        similar_pairs_dialog::message::Message::MediaItemDoubleClicked(path) => {
                            let media_focus_dialog =
                                media_focus_dialog::MediaFocusDialog::new(path);
                            let dialog = Dialog::MediaFocusDialog(media_focus_dialog.clone());
                            let default_task = media_focus_dialog.default_task();

                            self.dialog = Some(dialog);

                            return Task::batch([
                                task,
                                default_task.map(Message::MediaFocusDialogMessage),
                            ]);
                        }
                        _ => (),
                    }
                    return task;
                }
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
            Message::FileManagerShow(path) => {
                let _ = file_handle::FileHandle::show(&path);
                Task::none()
            }
            Message::DialogClose => {
                self.dialog = None;
                Task::none()
            }
            Message::CursorMove(point) => {
                match self.context_menu {
                    ContextMenu::None => self.context_menu_point = point,
                    _ => (),
                };
                Task::none()
            }
        }
    }
}
