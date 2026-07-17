pub mod message;
mod update;
mod view;

use arama_ai::model::model_container::wav2vec2;
use arama_env::ffmpeg_location::FfmpegLocationPreference;
use arama_sidecar::media::video::video_engine::discovery::FfmpegDiscoveryOutcome;

#[derive(Clone, Debug)]
pub struct AiSettings {
    message: String,
    ffmpeg_state: FfmpegState,
    ffmpeg_preference: FfmpegLocationPreference,
    candidate_failure: Option<(std::path::PathBuf, FfmpegState)>,
    ffmpeg_select_enabled: bool,
    wav2vec2_state: ModelCapabilityState,
    wav2vec2_generation: u64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum FfmpegState {
    #[default]
    Unknown,
    Checking,
    Ready,
    Missing,
    InvalidPair,
    ProbeTimedOut,
    SearchLimited,
    LegacyExcluded,
    InvalidSearchPath,
    FilesystemUnavailable,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ModelCapabilityState {
    Ready,
    Missing,
    Downloading,
    Errored(String),
}

impl Default for AiSettings {
    fn default() -> Self {
        Self {
            message: String::new(),
            ffmpeg_state: FfmpegState::Unknown,
            ffmpeg_preference: FfmpegLocationPreference::Auto,
            candidate_failure: None,
            ffmpeg_select_enabled: true,
            wav2vec2_state: if wav2vec2::model().ready().unwrap_or(false) {
                ModelCapabilityState::Ready
            } else {
                ModelCapabilityState::Missing
            },
            wav2vec2_generation: 0,
        }
    }
}

impl AiSettings {
    pub(crate) fn new(preference: FfmpegLocationPreference) -> Self {
        Self {
            ffmpeg_preference: preference,
            ..Self::default()
        }
    }

    pub(crate) fn should_check_ffmpeg(&self) -> bool {
        self.ffmpeg_state != FfmpegState::Checking
    }

    pub(crate) fn set_ffmpeg_checking(&mut self, preference: FfmpegLocationPreference) {
        self.ffmpeg_preference = preference;
        self.ffmpeg_state = FfmpegState::Checking;
        self.candidate_failure = None;
        self.message = arama_i18n::t("settings.ai.ffmpeg_checking");
    }

    pub(crate) fn set_ffmpeg_outcome(
        &mut self,
        preference: FfmpegLocationPreference,
        outcome: &FfmpegDiscoveryOutcome,
    ) {
        self.ffmpeg_preference = preference;
        self.ffmpeg_state = state_from_outcome(outcome);
        self.candidate_failure = None;
        self.message = if self.ffmpeg_state == FfmpegState::Ready {
            arama_i18n::t("settings.ai.ffmpeg_ready")
        } else {
            String::new()
        };
    }

    pub(crate) fn set_ffmpeg_ready(&mut self, preference: FfmpegLocationPreference, ready: bool) {
        self.ffmpeg_preference = preference;
        self.ffmpeg_state = if ready {
            FfmpegState::Ready
        } else {
            FfmpegState::Missing
        };
        self.candidate_failure = None;
        self.message = if ready {
            arama_i18n::t("settings.ai.ffmpeg_ready")
        } else {
            String::new()
        };
    }

    pub(crate) fn set_ffmpeg_candidate_failure(
        &mut self,
        preference: &FfmpegLocationPreference,
        outcome: &FfmpegDiscoveryOutcome,
    ) {
        let FfmpegLocationPreference::SelectedDirectory(directory) = preference else {
            return;
        };
        self.candidate_failure = Some((directory.clone(), state_from_outcome(outcome)));
    }

    pub(crate) fn set_ffmpeg_candidate_checking(&mut self, preference: &FfmpegLocationPreference) {
        let FfmpegLocationPreference::SelectedDirectory(directory) = preference else {
            return;
        };
        self.candidate_failure = Some((directory.clone(), FfmpegState::Checking));
    }

    pub(crate) fn set_ffmpeg_draining(&mut self, preference: FfmpegLocationPreference) {
        self.ffmpeg_preference = preference;
        self.ffmpeg_state = FfmpegState::Checking;
        self.message = arama_i18n::t("settings.ai.ffmpeg_draining");
    }

    pub(crate) fn set_ffmpeg_select_enabled(&mut self, enabled: bool) {
        self.ffmpeg_select_enabled = enabled;
    }
}

fn state_from_outcome(outcome: &FfmpegDiscoveryOutcome) -> FfmpegState {
    match outcome {
        FfmpegDiscoveryOutcome::Ready { .. } => FfmpegState::Ready,
        FfmpegDiscoveryOutcome::Missing => FfmpegState::Missing,
        FfmpegDiscoveryOutcome::InvalidPair(_) => FfmpegState::InvalidPair,
        FfmpegDiscoveryOutcome::ProbeTimedOut => FfmpegState::ProbeTimedOut,
        FfmpegDiscoveryOutcome::SearchLimitReached(_) => FfmpegState::SearchLimited,
        FfmpegDiscoveryOutcome::LegacyLocationExcluded => FfmpegState::LegacyExcluded,
        FfmpegDiscoveryOutcome::InvalidSearchPath => FfmpegState::InvalidSearchPath,
        FfmpegDiscoveryOutcome::FilesystemUnavailable(_) => FfmpegState::FilesystemUnavailable,
    }
}

#[cfg(test)]
mod presentation_tests {
    use std::path::PathBuf;

    use arama_sidecar::media::video::video_engine::discovery::{FfmpegDiscoveryOutcome, PairIssue};

    use super::{AiSettings, FfmpegLocationPreference, FfmpegState};

    #[test]
    fn candidate_failure_keeps_the_exact_prior_typed_authority_view() {
        let prior = FfmpegLocationPreference::SelectedDirectory(PathBuf::from("/prior"));
        let candidate = FfmpegLocationPreference::SelectedDirectory(PathBuf::from("/candidate"));
        let mut settings = AiSettings::new(prior.clone());
        settings.set_ffmpeg_outcome(
            prior.clone(),
            &FfmpegDiscoveryOutcome::LegacyLocationExcluded,
        );
        settings.set_ffmpeg_candidate_failure(
            &candidate,
            &FfmpegDiscoveryOutcome::InvalidPair(PairIssue::MissingMember),
        );

        assert_eq!(settings.ffmpeg_preference, prior);
        assert_eq!(settings.ffmpeg_state, FfmpegState::LegacyExcluded);
        assert_eq!(
            settings.candidate_failure,
            Some((PathBuf::from("/candidate"), FfmpegState::InvalidPair))
        );
    }
}
