pub mod message;
mod update;
pub mod util;
mod view;

use std::io::Result;

use crate::components::setup::downloader::{Downloader, config::DownloaderConfig};
use arama_ai::model::model_container::{clip, wav2vec2};
use arama_sidecar::media::video::video_engine::FfmpegDistribution;

pub struct Setup {
    pub finished: bool,
    ready: bool,
    downloader: Downloader,
}

impl Setup {
    /// Create a fallback Setup that reports itself as already finished,
    /// bypassing the setup wizard. Used when `Setup::default()` fails.
    pub fn fallback() -> Self {
        Self {
            finished: true,
            ready: true,
            downloader: Downloader::new(vec![]),
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn default() -> Result<Self> {
        let distribution = FfmpegDistribution::External;
        let downloader = Downloader::new_with_distribution(setup_configs(), distribution);
        let ready = downloader.requirements_ready();
        Ok(Self {
            finished: false,
            ready,
            downloader,
        })
    }

    pub fn initial_task(&self) -> iced::Task<message::Message> {
        self.downloader
            .initial_task()
            .map(message::Message::DownloaderMessage)
    }

    pub fn ready(&self) -> bool {
        self.ready
    }

    pub fn set_ffmpeg_ready(&mut self, ready: bool) {
        self.downloader.set_external_ffmpeg_ready(ready);
    }

    pub fn set_ffmpeg_checking(&mut self) {
        self.downloader.set_external_ffmpeg_checking();
    }

    pub fn set_ffmpeg_draining(&mut self) {
        self.downloader.set_external_ffmpeg_draining();
    }
}

fn setup_configs() -> Vec<DownloaderConfig> {
    vec![
        DownloaderConfig::AiModel(clip::model()),
        DownloaderConfig::AiModel(wav2vec2::model()),
        DownloaderConfig::Ffmepg,
    ]
}

#[cfg(test)]
mod tests {
    use arama_ai::model::model_container::{clip, wav2vec2};
    use arama_sidecar::media::video::video_engine::FfmpegDistribution;

    use crate::components::setup::downloader::{
        Downloader,
        config::DownloaderConfig,
        message as downloader_message,
        state::{DownloadProgress, DownloadState},
    };

    use super::{Setup, message::Message, setup_configs};

    fn macos_setup(
        clip_state: DownloadState,
        wav2vec2_state: DownloadState,
        is_downloading: bool,
    ) -> Setup {
        let states = setup_configs()
            .into_iter()
            .map(|config| {
                let state = match &config {
                    DownloaderConfig::AiModel(model) if model.name() == clip::model().name() => {
                        clip_state.clone()
                    }
                    DownloaderConfig::AiModel(model)
                        if model.name() == wav2vec2::model().name() =>
                    {
                        wav2vec2_state.clone()
                    }
                    DownloaderConfig::Ffmepg => DownloadState::ExternalRequired,
                    DownloaderConfig::AiModel(_) => unreachable!("unexpected setup model"),
                };
                (config, state)
            })
            .collect();
        let downloader =
            Downloader::from_states_for_test(FfmpegDistribution::External, states, is_downloading);
        let ready = downloader.requirements_ready();
        Setup {
            finished: false,
            ready,
            downloader,
        }
    }

    #[test]
    fn reconstructed_macos_setup_does_not_reopen_when_only_clip_is_ready() {
        let setup = macos_setup(DownloadState::NotRequired, DownloadState::Idle, false);

        assert!(setup.ready());
        assert!(!setup.finished);
    }

    #[test]
    fn production_setup_inventory_keeps_optional_audio_and_external_video() {
        let configs = setup_configs();

        assert!(configs.iter().any(
            |config| matches!(config, DownloaderConfig::AiModel(model) if model.name() == wav2vec2::model().name())
        ));
        assert!(
            configs
                .iter()
                .any(|config| matches!(config, DownloaderConfig::Ffmepg))
        );
    }

    #[test]
    fn failed_required_clip_download_remains_in_setup() {
        let mut setup = macos_setup(DownloadState::Downloading(10.0), DownloadState::Idle, true);

        let _ = setup.update(Message::DownloaderMessage(
            downloader_message::Message::AiModelProgressUpdated(
                0,
                DownloadProgress::Errored("checksum mismatch".to_owned()),
            ),
        ));

        assert!(!setup.ready());
        assert!(!setup.finished);
    }

    #[test]
    fn successful_clip_download_completes_macos_setup_without_audio_or_video() {
        let mut setup = macos_setup(DownloadState::Downloading(90.0), DownloadState::Idle, true);

        let _ = setup.update(Message::DownloaderMessage(
            downloader_message::Message::AiModelProgressUpdated(
                0,
                DownloadProgress::Finished(DownloaderConfig::AiModel(clip::model())),
            ),
        ));

        assert!(setup.ready());
        assert!(setup.finished);
    }

    #[test]
    fn ready_clip_with_housekeeping_warning_still_completes_setup() {
        let mut setup = macos_setup(DownloadState::Downloading(90.0), DownloadState::Idle, true);

        let _ = setup.update(Message::DownloaderMessage(
            downloader_message::Message::AiModelProgressUpdated(
                0,
                DownloadProgress::Finished(DownloaderConfig::AiModel(clip::model())),
            ),
        ));

        assert!(setup.ready());
        assert!(setup.finished);
    }

    #[test]
    fn optional_audio_failure_does_not_block_successful_clip_completion() {
        let mut setup = macos_setup(
            DownloadState::Downloading(90.0),
            DownloadState::Errored("optional download failed".to_owned()),
            true,
        );

        let _ = setup.update(Message::DownloaderMessage(
            downloader_message::Message::AiModelProgressUpdated(
                0,
                DownloadProgress::Finished(DownloaderConfig::AiModel(clip::model())),
            ),
        ));

        assert!(setup.ready());
        assert!(setup.finished);
    }

    #[test]
    fn explicit_skip_is_the_only_not_ready_bypass() {
        let mut setup = macos_setup(DownloadState::Idle, DownloadState::Idle, false);

        let _ = setup.update(Message::Skip);

        assert!(!setup.ready());
        assert!(setup.finished);
    }
}
