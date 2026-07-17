use std::path::{Path, PathBuf};

use arama_env::{Settings, ffmpeg_location::FfmpegLocationPreference};

use super::{DiscoverySource, FfmpegDiscoveryFailure, FfmpegDiscoveryOutcome};
use crate::media::video::video_engine::FfmpegToolchain;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PreferenceRetainReason {
    InvalidSelection,
    PersistencePreflight,
    SaveFailure,
    PickerCancelled,
    SettingsAuthorityMismatch,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PreferenceTransition {
    PublishedReady {
        preference: FfmpegLocationPreference,
        outcome: FfmpegDiscoveryOutcome,
    },
    PublishedAuto,
    Retained {
        preference: FfmpegLocationPreference,
        reason: PreferenceRetainReason,
        candidate_outcome: Option<FfmpegDiscoveryOutcome>,
    },
}

/// A persistable selected-directory candidate. Validation consumes this value,
/// preventing a caller from swapping in a different path after probing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectionCandidate {
    preference: FfmpegLocationPreference,
}

impl SelectionCandidate {
    pub fn preference(&self) -> &FfmpegLocationPreference {
        &self.preference
    }

    #[allow(
        dead_code,
        reason = "used by the private validated-selection constructor staged before locator wiring"
    )]
    fn directory(&self) -> &Path {
        let FfmpegLocationPreference::SelectedDirectory(directory) = &self.preference else {
            unreachable!("selection candidates always contain a selected directory")
        };
        directory
    }
}

/// Authority produced only by validating the tool pair inside one consumed
/// [`SelectionCandidate`]. Production callers cannot construct or modify it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidatedSelection {
    preference: FfmpegLocationPreference,
    toolchain: FfmpegToolchain,
}

impl ValidatedSelection {
    #[allow(
        dead_code,
        reason = "private authority constructor staged before production locator wiring"
    )]
    pub(super) fn bind(
        candidate: SelectionCandidate,
        toolchain: FfmpegToolchain,
    ) -> Result<Self, SelectionCandidate> {
        let directory = candidate.directory();
        let pair_is_bound = toolchain.ffmpeg_path().parent() == Some(directory)
            && toolchain.ffprobe_path().parent() == Some(directory);
        if !pair_is_bound {
            return Err(candidate);
        }
        Ok(Self {
            preference: candidate.preference,
            toolchain,
        })
    }

    pub(super) fn outcome(&self) -> FfmpegDiscoveryOutcome {
        FfmpegDiscoveryOutcome::Ready {
            toolchain: self.toolchain.clone(),
            source: DiscoverySource::SelectedDirectory,
        }
    }
}

pub fn prepare_selection(
    current: &FfmpegLocationPreference,
    directory: Option<PathBuf>,
) -> Result<SelectionCandidate, PreferenceTransition> {
    let Some(directory) = directory else {
        return Err(retained(
            current,
            PreferenceRetainReason::PickerCancelled,
            None,
        ));
    };
    let preference = FfmpegLocationPreference::SelectedDirectory(directory);
    if preference.validate_persistable().is_err() {
        return Err(retained(
            current,
            PreferenceRetainReason::PersistencePreflight,
            Some(FfmpegDiscoveryOutcome::InvalidSearchPath),
        ));
    }
    Ok(SelectionCandidate { preference })
}

pub fn reject_selection(
    current: &FfmpegLocationPreference,
    _candidate: SelectionCandidate,
    failure: FfmpegDiscoveryFailure,
) -> PreferenceTransition {
    retained(
        current,
        PreferenceRetainReason::InvalidSelection,
        Some(failure.into_outcome()),
    )
}

/// Save the exact full Settings value containing the validated preference, then
/// atomically publish that same preference and its already validated toolchain.
pub fn publish_validated_selection<E>(
    settings: &mut Settings,
    current: &FfmpegLocationPreference,
    validated: ValidatedSelection,
    save: impl FnOnce(&Settings) -> Result<(), E>,
) -> PreferenceTransition {
    if settings.ffmpeg_location != *current {
        return retained(
            current,
            PreferenceRetainReason::SettingsAuthorityMismatch,
            None,
        );
    }

    let prior = settings.ffmpeg_location.clone();
    settings.ffmpeg_location = validated.preference.clone();
    if save(settings).is_err() {
        settings.ffmpeg_location = prior;
        return retained(current, PreferenceRetainReason::SaveFailure, None);
    }

    PreferenceTransition::PublishedReady {
        preference: validated.preference,
        outcome: FfmpegDiscoveryOutcome::Ready {
            toolchain: validated.toolchain,
            source: DiscoverySource::SelectedDirectory,
        },
    }
}

pub fn clear_selection<E>(
    settings: &mut Settings,
    current: &FfmpegLocationPreference,
    save: impl FnOnce(&Settings) -> Result<(), E>,
) -> PreferenceTransition {
    if settings.ffmpeg_location != *current {
        return retained(
            current,
            PreferenceRetainReason::SettingsAuthorityMismatch,
            None,
        );
    }

    let prior = settings.ffmpeg_location.clone();
    settings.ffmpeg_location = FfmpegLocationPreference::Auto;
    if save(settings).is_err() {
        settings.ffmpeg_location = prior;
        return retained(current, PreferenceRetainReason::SaveFailure, None);
    }
    PreferenceTransition::PublishedAuto
}

fn retained(
    current: &FfmpegLocationPreference,
    reason: PreferenceRetainReason,
    candidate_outcome: Option<FfmpegDiscoveryOutcome>,
) -> PreferenceTransition {
    PreferenceTransition::Retained {
        preference: current.clone(),
        reason,
        candidate_outcome,
    }
}

#[cfg(test)]
mod tests;
