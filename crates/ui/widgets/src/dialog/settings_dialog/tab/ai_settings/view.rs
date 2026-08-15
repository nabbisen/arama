use arama_ai::model::model_container::clip;
use arama_env::ffmpeg_location::FfmpegLocationPreference;
use arama_i18n::t;
use iced::{
    Element,
    widget::{button, column, container, space, text},
};

use super::{AiSettings, FfmpegState, ModelCapabilityState, message::Message};

impl AiSettings {
    pub fn view(&self) -> Element<'_, Message> {
        let clip: Element<Message> = if clip::model().ready().unwrap_or(false) {
            text(t("settings.ai.clip_ready")).into()
        } else {
            column![
                text(t("settings.ai.clip_missing")),
                button(text(t("settings.ai.clip_load"))).on_press(Message::LoadStart),
            ]
            .into()
        };

        let authority = match &self.ffmpeg_preference {
            FfmpegLocationPreference::Auto => t("settings.ai.ffmpeg_mode_auto"),
            FfmpegLocationPreference::SelectedDirectory(directory) => format!(
                "{}: {}",
                t("settings.ai.ffmpeg_mode_selected"),
                directory.display()
            ),
        };
        let ffmpeg_status: Element<Message> = match self.ffmpeg_state {
            FfmpegState::Unknown | FfmpegState::Checking => {
                text(t("settings.ai.ffmpeg_checking")).into()
            }
            FfmpegState::Ready => text(t("settings.ai.ffmpeg_ready")).into(),
            FfmpegState::Missing => column![
                text(t("settings.ai.ffmpeg_external")),
                button(text(t("settings.ai.ffmpeg_recheck"))).on_press(Message::CheckFfmpeg),
            ]
            .into(),
            state => column![
                text(ffmpeg_failure_message(
                    state,
                    matches!(self.ffmpeg_preference, FfmpegLocationPreference::Auto)
                )),
                button(text(t("settings.ai.ffmpeg_recheck"))).on_press(Message::CheckFfmpeg),
            ]
            .into(),
        };
        let candidate_failure: Element<Message> =
            if let Some((directory, state)) = &self.candidate_failure {
                if *state == FfmpegState::Checking {
                    text(format!(
                        "{}: {}",
                        t("settings.ai.ffmpeg_candidate_checking"),
                        directory.display()
                    ))
                    .into()
                } else {
                    text(format!(
                        "{}: {} — {}",
                        t("settings.ai.ffmpeg_candidate_rejected"),
                        directory.display(),
                        // This branch only ever runs for a Selected-directory
                        // candidate check (set_ffmpeg_candidate_failure is
                        // only called with a SelectedDirectory preference),
                        // so is_auto is always false here.
                        ffmpeg_failure_message(*state, false)
                    ))
                    .into()
                }
            } else {
                space().into()
            };
        let select_button = button(text(t("settings.ai.ffmpeg_select")));
        let select_button = if self.ffmpeg_select_enabled {
            select_button.on_press(Message::SelectFfmpegDirectory)
        } else {
            select_button
        };
        let ffmpeg = column![
            text(authority),
            ffmpeg_status,
            candidate_failure,
            select_button,
            button(text(t("settings.ai.ffmpeg_clear"))).on_press(Message::ClearFfmpegSelection),
        ]
        .spacing(6);

        let wav2vec2: Element<Message> = match &self.wav2vec2_state {
            ModelCapabilityState::Ready => text(t("settings.ai.wav2vec2_ready")).into(),
            ModelCapabilityState::Missing => column![
                text(t("settings.ai.wav2vec2_missing")),
                button(text(t("settings.ai.wav2vec2_get"))).on_press(Message::GetWav2vec2Start),
            ]
            .into(),
            ModelCapabilityState::Downloading => text(t("settings.ai.wav2vec2_downloading")).into(),
            ModelCapabilityState::Errored(error) => column![
                text(format!("{}: {error}", t("settings.ai.wav2vec2_error"))),
                button(text(t("settings.ai.wav2vec2_retry"))).on_press(Message::GetWav2vec2Start),
            ]
            .into(),
        };

        let message = if !self.message.is_empty() {
            container(text(self.message.to_owned()))
        } else {
            container(space())
        };

        column![clip, wav2vec2, ffmpeg, message].spacing(12).into()
    }
}

/// Task 019: `FilesystemUnavailable` needs different text depending on which
/// discovery mode produced it. "Check permissions on this folder" is sound
/// advice when the user picked one directory (Selected mode); it does not
/// fit Auto mode, where there is no single folder and the cause is a
/// filesystem error somewhere in a `PATH` scan. `is_auto` selects between
/// them; every other state's message is mode-independent.
fn ffmpeg_failure_message(state: FfmpegState, is_auto: bool) -> String {
    t(match state {
        FfmpegState::InvalidPair => "settings.ai.ffmpeg_invalid_pair",
        FfmpegState::ProbeTimedOut => "settings.ai.ffmpeg_probe_timed_out",
        FfmpegState::SearchLimited => "settings.ai.ffmpeg_search_limited",
        FfmpegState::LegacyExcluded => "settings.ai.ffmpeg_legacy_excluded",
        FfmpegState::InvalidSearchPath => "settings.ai.ffmpeg_invalid_path",
        FfmpegState::FilesystemUnavailable if is_auto => {
            "settings.ai.ffmpeg_filesystem_unavailable_auto"
        }
        FfmpegState::FilesystemUnavailable => "settings.ai.ffmpeg_filesystem_unavailable_selected",
        FfmpegState::Missing => "settings.ai.ffmpeg_external",
        FfmpegState::Unknown | FfmpegState::Checking => "settings.ai.ffmpeg_checking",
        FfmpegState::Ready => "settings.ai.ffmpeg_ready",
    })
}
