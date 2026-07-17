use arama_ai::model::model_container::{ModelDownloadStatus, clip, wav2vec2};
use arama_i18n::t;
use iced::Task;

use super::{AiSettings, message::Message};

fn wav2vec2_state(status: ModelDownloadStatus) -> super::ModelCapabilityState {
    match status {
        ModelDownloadStatus::Ready => super::ModelCapabilityState::Ready,
        ModelDownloadStatus::Downloading => super::ModelCapabilityState::Downloading,
        ModelDownloadStatus::Failed => {
            super::ModelCapabilityState::Errored(t("settings.ai.wav2vec2_error"))
        }
        ModelDownloadStatus::Idle => super::ModelCapabilityState::Missing,
    }
}

impl AiSettings {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::LoadStart => {
                self.message = t("settings.ai.clip_loading");

                Task::perform(
                    async {
                        match clip::model().download().await {
                            Ok(_) => None,
                            Err(err) => Some(err.to_string()),
                        }
                    },
                    Message::Loaded,
                )
            }
            Message::Loaded(result) => {
                if let Some(err) = result {
                    self.message = err;
                }
                Task::none()
            }
            Message::RefreshCapabilities => {
                self.wav2vec2_state = wav2vec2_state(wav2vec2::model().download_status());
                Task::none()
            }
            Message::GetWav2vec2Start => {
                if self.wav2vec2_state == super::ModelCapabilityState::Downloading
                    || wav2vec2::model().download_status() == ModelDownloadStatus::Downloading
                {
                    self.wav2vec2_state = super::ModelCapabilityState::Downloading;
                    return Task::none();
                }
                self.wav2vec2_generation = self.wav2vec2_generation.saturating_add(1);
                let generation = self.wav2vec2_generation;
                self.wav2vec2_state = super::ModelCapabilityState::Downloading;
                Task::perform(
                    async {
                        wav2vec2::model()
                            .download()
                            .await
                            .map_err(|error| error.to_string())
                    },
                    move |result| Message::Wav2vec2Got(generation, result),
                )
            }
            Message::Wav2vec2Got(generation, result) => {
                if generation != self.wav2vec2_generation {
                    return Task::none();
                }
                self.wav2vec2_state = match result {
                    Ok(()) => super::ModelCapabilityState::Ready,
                    Err(error) => super::ModelCapabilityState::Errored(error),
                };
                Task::none()
            }
            Message::CheckFfmpeg => {
                if self.ffmpeg_state == super::FfmpegState::Checking {
                    return Task::none();
                }
                self.ffmpeg_state = super::FfmpegState::Checking;
                self.message = t("settings.ai.ffmpeg_checking");
                Task::done(Message::FfmpegRecheckRequested)
            }
            Message::FfmpegRecheckRequested
            | Message::SelectFfmpegDirectory
            | Message::ClearFfmpegSelection => Task::none(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::wav2vec2_state;
    use crate::dialog::settings_dialog::tab::ai_settings::{
        AiSettings, FfmpegState, ModelCapabilityState, message::Message,
    };
    use arama_ai::model::model_container::ModelDownloadStatus;
    use arama_env::ffmpeg_location::FfmpegLocationPreference;

    fn external_settings(state: FfmpegState) -> AiSettings {
        AiSettings {
            message: String::new(),
            ffmpeg_state: state,
            ffmpeg_preference: FfmpegLocationPreference::Auto,
            candidate_failure: None,
            ffmpeg_select_enabled: true,
            wav2vec2_state: ModelCapabilityState::Missing,
            wav2vec2_generation: 0,
        }
    }

    #[test]
    fn external_ffmpeg_action_requests_app_discovery() {
        let mut settings = external_settings(FfmpegState::Missing);
        let _ = settings.update(Message::CheckFfmpeg);
        assert_eq!(settings.ffmpeg_state, FfmpegState::Checking);
    }

    #[test]
    fn duplicate_check_is_ignored_while_one_is_pending() {
        let mut settings = external_settings(FfmpegState::Checking);
        let _ = settings.update(Message::CheckFfmpeg);
        assert_eq!(settings.ffmpeg_state, FfmpegState::Checking);
        assert!(settings.message.is_empty());
    }

    #[test]
    fn optional_audio_missing_and_error_states_offer_download_or_retry() {
        let mut settings = external_settings(FfmpegState::Ready);
        assert_eq!(settings.wav2vec2_state, ModelCapabilityState::Missing);

        let _ = settings.update(Message::GetWav2vec2Start);
        assert_eq!(settings.wav2vec2_state, ModelCapabilityState::Downloading);

        let _ = settings.update(Message::Wav2vec2Got(1, Err("network error".to_owned())));
        assert!(matches!(
            settings.wav2vec2_state,
            ModelCapabilityState::Errored(_)
        ));

        let _ = settings.update(Message::GetWav2vec2Start);
        assert_eq!(settings.wav2vec2_state, ModelCapabilityState::Downloading);
    }

    #[test]
    fn stale_optional_audio_completion_is_ignored() {
        let mut settings = external_settings(FfmpegState::Ready);
        settings.wav2vec2_generation = 2;
        settings.wav2vec2_state = ModelCapabilityState::Downloading;

        let _ = settings.update(Message::Wav2vec2Got(1, Ok(())));

        assert_eq!(settings.wav2vec2_state, ModelCapabilityState::Downloading);
    }

    #[test]
    fn duplicate_optional_audio_start_is_coalesced_locally() {
        let mut settings = external_settings(FfmpegState::Ready);

        let _ = settings.update(Message::GetWav2vec2Start);
        let generation = settings.wav2vec2_generation;
        let _ = settings.update(Message::GetWav2vec2Start);

        assert_eq!(settings.wav2vec2_generation, generation);
        assert_eq!(settings.wav2vec2_state, ModelCapabilityState::Downloading);
    }

    #[test]
    fn shared_download_lifecycle_maps_to_settings_states() {
        assert_eq!(
            wav2vec2_state(ModelDownloadStatus::Idle),
            ModelCapabilityState::Missing
        );
        assert_eq!(
            wav2vec2_state(ModelDownloadStatus::Downloading),
            ModelCapabilityState::Downloading
        );
        assert_eq!(
            wav2vec2_state(ModelDownloadStatus::Ready),
            ModelCapabilityState::Ready
        );
        assert!(matches!(
            wav2vec2_state(ModelDownloadStatus::Failed),
            ModelCapabilityState::Errored(_)
        ));
    }
}
