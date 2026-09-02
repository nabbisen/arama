use iced::Task;

use crate::dialog::settings_dialog::tab::general_settings;

use super::{SettingsDialog, message::Message};

impl SettingsDialog {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::TargetMediaTypeChanged(_)
            | Message::SubDirDepthLimitChanged(_)
            | Message::SimilarityThresholdChanged(_)
            | Message::LocaleChanged(_)
            | Message::ThemeChanged(_)
            | Message::FfmpegRecheckRequested
            | Message::FfmpegSelectRequested
            | Message::FfmpegClearRequested
            | Message::CacheDeleteRequested => Task::none(),
            Message::RefreshAiCapabilities => self
                .ai_settings
                .update(super::tab::ai_settings::message::Message::RefreshCapabilities)
                .map(Message::AiSettingsTabMessage),
            Message::TabSelect(tab) => {
                let check_ffmpeg = matches!(tab, super::Tab::Ai);
                self.tab = tab;
                if check_ffmpeg {
                    let refresh = self
                        .ai_settings
                        .update(super::tab::ai_settings::message::Message::RefreshCapabilities)
                        .map(Message::AiSettingsTabMessage);
                    let ffmpeg = if self.ai_settings.should_check_ffmpeg() {
                        self.ai_settings
                            .update(super::tab::ai_settings::message::Message::CheckFfmpeg)
                            .map(Message::AiSettingsTabMessage)
                    } else {
                        Task::none()
                    };
                    Task::batch([refresh, ffmpeg])
                } else {
                    Task::none()
                }
            }
            Message::GeneralSettingsTabMessage(message) => {
                let task = self
                    .general_settings
                    .update(message.clone())
                    .map(Message::GeneralSettingsTabMessage);

                match message {
                    general_settings::message::Message::TargetMediaTypeChanged(x) => {
                        Task::batch([task, Task::done(Message::TargetMediaTypeChanged(x))])
                    }
                    general_settings::message::Message::SubDirDepthLimitChanged(x) => {
                        Task::batch([task, Task::done(Message::SubDirDepthLimitChanged(x))])
                    }
                    general_settings::message::Message::SimilarityThresholdChanged(v) => {
                        Task::batch([task, Task::done(Message::SimilarityThresholdChanged(v))])
                    }
                    general_settings::message::Message::LocaleChanged(l) => {
                        Task::batch([task, Task::done(Message::LocaleChanged(l))])
                    }
                    general_settings::message::Message::ThemeChanged(t) => {
                        Task::batch([task, Task::done(Message::ThemeChanged(t))])
                    }
                }
            }
            Message::AiSettingsTabMessage(message) => {
                let event = match message {
                    super::tab::ai_settings::message::Message::FfmpegRecheckRequested => {
                        Some(Message::FfmpegRecheckRequested)
                    }
                    super::tab::ai_settings::message::Message::SelectFfmpegDirectory => {
                        Some(Message::FfmpegSelectRequested)
                    }
                    super::tab::ai_settings::message::Message::ClearFfmpegSelection => {
                        Some(Message::FfmpegClearRequested)
                    }
                    _ => None,
                };
                let task = self
                    .ai_settings
                    .update(message)
                    .map(Message::AiSettingsTabMessage);
                if let Some(event) = event {
                    Task::batch([task, Task::done(event)])
                } else {
                    task
                }
            }
            Message::FileSystemSettingsTabMessage(message) => {
                let event = matches!(
                    message,
                    super::tab::file_system_settings::message::Message::CacheDeleteRequested
                )
                .then_some(Message::CacheDeleteRequested);
                let task = self
                    .file_system_settings
                    .update(message)
                    .map(Message::FileSystemSettingsTabMessage);
                if let Some(event) = event {
                    Task::batch([task, Task::done(event)])
                } else {
                    task
                }
            }
            Message::AboutTabMessage(message) => {
                self.about.update(message).map(Message::AboutTabMessage)
            }
        }
    }
}
