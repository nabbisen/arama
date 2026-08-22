use arama_i18n::{set_locale, t};
use arama_ui_layout::{aside, footer, header};
use arama_ui_main::{components::gallery::image_cell, views::gallery};
use arama_ui_widgets::dialog::{media_focus_dialog, settings_dialog, similar_pairs_dialog};
use iced::Task;

use super::super::{
    App, Dialog,
    message::{FfmpegRequestIntent, Message},
    setup_became_complete, setup_complete,
};
use arama_ui_widgets::context_menu::ContextMenuState;

fn setup_message_refreshes_ai(message: &arama_ui_main::views::setup::message::Message) -> bool {
    matches!(
        message,
        arama_ui_main::views::setup::message::Message::DownloaderMessage(
            arama_ui_main::components::setup::downloader::message::Message::AiModelProgressUpdated(
                _,
                _
            )
        )
    )
}

impl App {
    pub(super) fn handle_setup_message(
        &mut self,
        message: arama_ui_main::views::setup::message::Message,
    ) -> Task<Message> {
        if matches!(
            message,
            arama_ui_main::views::setup::message::Message::FfmpegRecheckRequested
        ) {
            return self.request_current_ffmpeg(FfmpegRequestIntent::Recheck);
        }
        if matches!(
            message,
            arama_ui_main::views::setup::message::Message::FfmpegSelectRequested
        ) {
            return self.pick_ffmpeg_directory();
        }
        let refresh_ai = setup_message_refreshes_ai(&message);
        let was_ready = setup_complete(self.setup.finished, self.setup.ready());
        let task = self
            .setup
            .update(message.clone())
            .map(Message::SetupMessage);
        let refresh_task = if refresh_ai {
            Task::done(Message::SettingsDialogMessage(
                settings_dialog::message::Message::RefreshAiCapabilities,
            ))
        } else {
            Task::none()
        };
        if setup_became_complete(was_ready, self.setup.finished, self.setup.ready()) {
            self.processing_on();
            Task::batch([task, refresh_task, Task::done(Message::CacheRequire(None))])
        } else {
            Task::batch([task, refresh_task])
        }
    }

    pub(super) fn handle_gallery_message(
        &mut self,
        message: gallery::message::Message,
    ) -> Task<Message> {
        let task = self
            .gallery
            .update(message.clone())
            .map(Message::GalleryMessage);

        match message {
            gallery::message::Message::FilterChanged(_)
            | gallery::message::Message::FilterClear => (),
            gallery::message::Message::ImageCellMessage(message) => match message {
                image_cell::message::Message::ImageCellEnter(path) => {
                    self.image_cell_path_update(Some(path));
                }
                image_cell::message::Message::ImageSelect => {
                    if let Some(path) = &self.image_cell_path {
                        let media_focus_dialog = media_focus_dialog::MediaFocusDialog::new(
                            path,
                            self.settings.cache_lookup_strategy,
                            self.settings.similarity_threshold,
                            self.ffmpeg_authority.toolchain().cloned(),
                        );
                        let dialog = Dialog::MediaFocusDialog(media_focus_dialog.clone());
                        let default_task = media_focus_dialog.default_task();

                        self.dialog = Some(dialog);

                        return Task::batch([
                            task,
                            default_task.map(Message::MediaFocusDialogMessage),
                        ]);
                    }
                }
                image_cell::message::Message::ContextMenuOpen => match self.context_menu.state {
                    ContextMenuState::None => {
                        if let Some(path) = &self.image_cell_path {
                            self.context_menu.state = ContextMenuState::ImageCell(path.to_owned())
                        }
                    }
                    _ => self.context_menu.state = ContextMenuState::None,
                },
            },
            gallery::message::Message::CursorExit => self.image_cell_path_update(None),
        }

        task
    }

    pub(super) fn handle_header_message(
        &mut self,
        message: header::message::Message,
    ) -> Task<Message> {
        let task = self
            .header
            .update(message.clone())
            .map(Message::HeaderMessage);

        match message {
            header::message::Message::Event(message) => match message {
                header::message::Event::DirSelect(path) => {
                    let expand_task = self.aside.update_dir_tree(&path);
                    return Task::batch([
                        self.on_dir_changed(path, task),
                        expand_task.map(Message::AsideMessage),
                    ]);
                }
                header::message::Event::SimilarPairsDialogOpen => {
                    let Some(dir_node) = self.dir_node.clone() else {
                        self.push_error_toast(
                            t("toast.similarity_pairs.title"),
                            t("toast.similarity_pairs.select_dir_first.body"),
                        );
                        return task;
                    };
                    let dialog = similar_pairs_dialog::SimilarPairsDialog::new(
                        dir_node,
                        None,
                        self.settings.similarity_threshold,
                        self.ffmpeg_authority.toolchain().cloned(),
                    );
                    self.dialog = Some(Dialog::SimilarPairsDialog(dialog.clone()));
                    return dialog
                        .default_task()
                        .map(Message::SimilarPairsDialogMessage);
                }
            },
            header::message::Message::Internal(_) => (),
        }

        task
    }

    pub(super) fn handle_aside_message(
        &mut self,
        message: aside::message::Message,
    ) -> Task<Message> {
        let task = self
            .aside
            .update(message.clone())
            .map(Message::AsideMessage);

        match message {
            aside::message::Message::Event(message) => match message {
                aside::message::Event::DirSelect(path) => {
                    return self.on_dir_changed(path, task);
                }
            },
            aside::message::Message::Internal(_) => (),
        }

        task
    }

    pub(super) fn handle_footer_message(
        &mut self,
        message: footer::message::Message,
    ) -> Task<Message> {
        let task = self
            .footer
            .update(message.clone())
            .map(Message::FooterMessage);

        if let footer::message::Message::ThumbnailSizeChanged(value) = message {
            self.thumbnail_size_update(value)
        }

        task
    }

    pub(super) fn handle_media_focus_dialog_message(
        &mut self,
        message: media_focus_dialog::message::Message,
    ) -> Task<Message> {
        if let Some(Dialog::MediaFocusDialog(x)) = &mut self.dialog {
            let task = x
                .update(message.clone())
                .map(Message::MediaFocusDialogMessage);

            match message {
                media_focus_dialog::message::Message::CloseClick => self.dialog = None,
                media_focus_dialog::message::Message::CacheLookupStrategyChanged(x) => {
                    self.settings.cache_lookup_strategy = x;
                    self.save_settings();
                }
                _ => (),
            }

            return task;
        }
        Task::none()
    }

    pub(super) fn handle_similar_pairs_dialog_message(
        &mut self,
        message: similar_pairs_dialog::message::Message,
    ) -> Task<Message> {
        if let Some(Dialog::SimilarPairsDialog(x)) = &mut self.dialog {
            let task = x
                .update(message.clone())
                .map(Message::SimilarPairsDialogMessage);

            if let similar_pairs_dialog::message::Message::MediaItemDoubleClicked(path) = message {
                let media_focus_dialog = media_focus_dialog::MediaFocusDialog::new(
                    path,
                    self.settings.cache_lookup_strategy,
                    self.settings.similarity_threshold,
                    self.ffmpeg_authority.toolchain().cloned(),
                );
                let dialog = Dialog::MediaFocusDialog(media_focus_dialog.clone());
                let default_task = media_focus_dialog.default_task();

                self.dialog = Some(dialog);

                return Task::batch([task, default_task.map(Message::MediaFocusDialogMessage)]);
            }

            return task;
        }
        Task::none()
    }

    pub(super) fn handle_settings_dialog_message(
        &mut self,
        message: settings_dialog::message::Message,
    ) -> Task<Message> {
        match &message {
            settings_dialog::message::Message::FfmpegRecheckRequested => {
                return self.request_current_ffmpeg(FfmpegRequestIntent::Recheck);
            }
            settings_dialog::message::Message::FfmpegSelectRequested => {
                return self.pick_ffmpeg_directory();
            }
            settings_dialog::message::Message::FfmpegClearRequested => {
                return self.clear_ffmpeg_selection();
            }
            _ => {}
        }
        let task = self
            .settings_page
            .update(message.clone())
            .map(Message::SettingsDialogMessage);

        match message {
            settings_dialog::message::Message::TargetMediaTypeChanged(x) => {
                self.settings.target_media_type = x;
                self.save_settings();

                return if self.processing {
                    task
                } else {
                    self.processing_on();
                    Task::batch([task, Task::done(Message::CacheRequire(None))])
                };
            }
            settings_dialog::message::Message::SubDirDepthLimitChanged(x) => {
                self.settings.sub_dir_depth_limit = x;
                self.save_settings();

                return if self.processing {
                    task
                } else {
                    self.processing_on();
                    Task::batch([task, Task::done(Message::CacheRequire(None))])
                };
            }
            settings_dialog::message::Message::SimilarityThresholdChanged(v) => {
                self.settings.similarity_threshold = v;
                self.save_settings();
            }
            settings_dialog::message::Message::LocaleChanged(l) => {
                self.settings.locale = l;
                set_locale(l);
                self.save_settings();
            }
            settings_dialog::message::Message::ThemeChanged(theme) => {
                self.settings.theme = theme;
                arama_theme::set_theme(theme);
                self.save_settings();
            }
            _ => (),
        }

        task
    }
}

#[cfg(test)]
mod tests {
    use arama_ui_main::{
        components::setup::downloader::{message as downloader_message, state::DownloadProgress},
        views::setup::message::Message as SetupMessage,
    };

    use super::setup_message_refreshes_ai;

    #[test]
    fn model_progress_refreshes_persistent_ai_settings_state() {
        let message =
            SetupMessage::DownloaderMessage(downloader_message::Message::AiModelProgressUpdated(
                1,
                DownloadProgress::Downloading(50.0),
            ));

        assert!(setup_message_refreshes_ai(&message));
        assert!(!setup_message_refreshes_ai(&SetupMessage::Skip));
    }
}
