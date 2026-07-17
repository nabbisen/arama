use std::{
    path::{Path, PathBuf},
    sync::atomic::Ordering,
    time::{Duration, Instant},
};

use arama_env::{ffmpeg_location::FfmpegLocationPreference, local_bin_dir};

use super::{
    CandidatePlanningStop, DiscoverySource, DiscoveryWork, FfmpegDiscoveryOutcome,
    FfmpegLocatorPolicy, FilesystemIssue, NormalizedCandidate, PairIssue, SearchLimit,
    StdCandidateFilesystem, ValidatedSelection, normalize_auto_candidates, prepare_selection,
};
use crate::media::video::video_engine::{
    FfmpegToolchain, PROBE_POLL_INTERVAL, ProbePolicy, bin_name, parse_version_token,
    run_bounded_probe_with_cancellation,
};

pub(super) enum WorkerCompletion {
    Outcome(FfmpegDiscoveryOutcome),
    SelectedReady(ValidatedSelection),
}

impl WorkerCompletion {
    pub(super) fn outcome(&self) -> FfmpegDiscoveryOutcome {
        match self {
            Self::Outcome(outcome) => outcome.clone(),
            Self::SelectedReady(validated) => validated.outcome(),
        }
    }
}

pub(super) fn run_discovery_work(
    work: DiscoveryWork,
    policy: FfmpegLocatorPolicy,
) -> WorkerCompletion {
    let started = Instant::now();
    let outcome = match &work.preference {
        FfmpegLocationPreference::Auto => {
            let path = std::env::var_os("PATH");
            let native = native_prefix();
            let mut control = || checkpoint(&work, started, policy.attempt_timeout);
            let mut filesystem = StdCandidateFilesystem;
            let planned = normalize_auto_candidates(
                path.as_deref(),
                native.as_deref(),
                policy,
                &mut control,
                &mut filesystem,
            );
            if let Some(stop) = planned.stop {
                return completion(stop_outcome(stop));
            }
            let mut failure = planned
                .filesystem_diagnostic
                .map(|diagnostic| FfmpegDiscoveryOutcome::FilesystemUnavailable(diagnostic.issue))
                .unwrap_or(FfmpegDiscoveryOutcome::Missing);
            for candidate in planned.candidates {
                match validate_candidate(&work, started, policy, candidate) {
                    ready @ FfmpegDiscoveryOutcome::Ready { .. } => return completion(ready),
                    outcome => failure = failure.prefer(outcome),
                }
            }
            if planned.raw_truncated || planned.candidate_truncated {
                failure = failure.prefer(FfmpegDiscoveryOutcome::SearchLimitReached(
                    SearchLimit::CandidateCount,
                ));
            }
            failure
        }
        FfmpegLocationPreference::SelectedDirectory(directory) => validate_candidate(
            &work,
            started,
            policy,
            NormalizedCandidate {
                directory: directory.clone(),
                source: DiscoverySource::SelectedDirectory,
            },
        ),
    };
    match (&work.preference, outcome) {
        (
            FfmpegLocationPreference::SelectedDirectory(directory),
            FfmpegDiscoveryOutcome::Ready { toolchain, .. },
        ) => {
            let Ok(candidate) =
                prepare_selection(&FfmpegLocationPreference::Auto, Some(directory.clone()))
            else {
                return WorkerCompletion::Outcome(FfmpegDiscoveryOutcome::InvalidSearchPath);
            };
            match ValidatedSelection::bind(candidate, toolchain) {
                Ok(validated) => WorkerCompletion::SelectedReady(validated),
                Err(_) => WorkerCompletion::Outcome(FfmpegDiscoveryOutcome::FilesystemUnavailable(
                    FilesystemIssue::MetadataOrIdentity,
                )),
            }
        }
        (_, outcome) => WorkerCompletion::Outcome(outcome),
    }
}

fn completion(outcome: FfmpegDiscoveryOutcome) -> WorkerCompletion {
    WorkerCompletion::Outcome(outcome)
}

fn validate_candidate(
    work: &DiscoveryWork,
    started: Instant,
    policy: FfmpegLocatorPolicy,
    candidate: NormalizedCandidate,
) -> FfmpegDiscoveryOutcome {
    if !candidate.directory.is_absolute() {
        return FfmpegDiscoveryOutcome::InvalidSearchPath;
    }
    if let Err(stop) = checkpoint(work, started, policy.attempt_timeout) {
        return stop_outcome(stop);
    }
    let canonical_directory = match std::fs::canonicalize(&candidate.directory) {
        Ok(path) => path,
        Err(error) => return filesystem_outcome(&error),
    };
    if let Err(stop) = checkpoint(work, started, policy.attempt_timeout) {
        return stop_outcome(stop);
    }
    match is_legacy_candidate(
        work,
        started,
        policy.attempt_timeout,
        &candidate.directory,
        &canonical_directory,
    ) {
        Ok(true) => return FfmpegDiscoveryOutcome::LegacyLocationExcluded,
        Ok(false) => {}
        Err(outcome) => return outcome,
    }

    let ffmpeg = candidate.directory.join(bin_name::FFMPEG);
    let ffprobe = candidate.directory.join(bin_name::FFPROBE);
    let mut presence = [false; 2];
    for (index, executable) in [&ffmpeg, &ffprobe].into_iter().enumerate() {
        if let Err(stop) = checkpoint(work, started, policy.attempt_timeout) {
            return stop_outcome(stop);
        }
        match std::fs::metadata(executable) {
            Ok(metadata) => presence[index] = metadata.is_file(),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return filesystem_outcome(&error),
        }
        if let Err(stop) = checkpoint(work, started, policy.attempt_timeout) {
            return stop_outcome(stop);
        }
    }
    if let Some(outcome) = incomplete_pair_outcome(presence) {
        return outcome;
    }

    let ffmpeg_version = match probe_version(work, started, policy, &ffmpeg, "ffmpeg") {
        Ok(version) => version,
        Err(outcome) => return outcome,
    };
    let ffprobe_version = match probe_version(work, started, policy, &ffprobe, "ffprobe") {
        Ok(version) => version,
        Err(outcome) => return outcome,
    };
    if ffmpeg_version != ffprobe_version {
        return FfmpegDiscoveryOutcome::InvalidPair(PairIssue::VersionMismatch);
    }
    FfmpegDiscoveryOutcome::Ready {
        toolchain: FfmpegToolchain { ffmpeg, ffprobe },
        source: candidate.source,
    }
}

fn probe_version(
    work: &DiscoveryWork,
    started: Instant,
    policy: FfmpegLocatorPolicy,
    executable: &Path,
    tool_name: &str,
) -> Result<String, FfmpegDiscoveryOutcome> {
    checkpoint(work, started, policy.attempt_timeout).map_err(stop_outcome)?;
    let remaining = policy
        .attempt_timeout
        .saturating_sub(started.elapsed())
        .min(policy.probe_timeout);
    if remaining.is_zero() {
        return Err(FfmpegDiscoveryOutcome::SearchLimitReached(
            SearchLimit::WholeAttempt,
        ));
    }
    let output = run_bounded_probe_with_cancellation(
        executable,
        ProbePolicy {
            timeout: remaining,
            poll_interval: PROBE_POLL_INTERVAL,
        },
        Some(&work.cancellation),
    )
    .map_err(|error| {
        let message = error.to_string();
        if work.is_cancelled() || message.contains("cancelled") {
            FfmpegDiscoveryOutcome::SearchLimitReached(SearchLimit::WorkerDraining)
        } else if message.contains("timed out") {
            FfmpegDiscoveryOutcome::ProbeTimedOut
        } else if message.contains("output exceeded") {
            FfmpegDiscoveryOutcome::InvalidPair(PairIssue::OutputLimit)
        } else {
            FfmpegDiscoveryOutcome::InvalidPair(PairIssue::MalformedVersion)
        }
    })?;
    checkpoint(work, started, policy.attempt_timeout).map_err(stop_outcome)?;
    parse_version_token(&output, tool_name)
        .map_err(|_| FfmpegDiscoveryOutcome::InvalidPair(PairIssue::MalformedVersion))
}

fn checkpoint(
    work: &DiscoveryWork,
    started: Instant,
    attempt_timeout: Duration,
) -> Result<(), CandidatePlanningStop> {
    if work.cancellation.load(Ordering::Acquire) {
        Err(CandidatePlanningStop::Cancelled)
    } else if started.elapsed() >= attempt_timeout {
        Err(CandidatePlanningStop::SearchLimit(
            SearchLimit::WholeAttempt,
        ))
    } else {
        Ok(())
    }
}

fn stop_outcome(stop: CandidatePlanningStop) -> FfmpegDiscoveryOutcome {
    match stop {
        CandidatePlanningStop::Cancelled => {
            FfmpegDiscoveryOutcome::SearchLimitReached(SearchLimit::WorkerDraining)
        }
        CandidatePlanningStop::SearchLimit(limit) => {
            FfmpegDiscoveryOutcome::SearchLimitReached(limit)
        }
    }
}

fn filesystem_outcome(error: &std::io::Error) -> FfmpegDiscoveryOutcome {
    FfmpegDiscoveryOutcome::FilesystemUnavailable(
        if error.kind() == std::io::ErrorKind::PermissionDenied {
            FilesystemIssue::Access
        } else {
            FilesystemIssue::MetadataOrIdentity
        },
    )
}

fn incomplete_pair_outcome(presence: [bool; 2]) -> Option<FfmpegDiscoveryOutcome> {
    match presence {
        [false, false] => Some(FfmpegDiscoveryOutcome::Missing),
        [true, false] | [false, true] => Some(FfmpegDiscoveryOutcome::InvalidPair(
            PairIssue::MissingMember,
        )),
        [true, true] => None,
    }
}

fn is_legacy_candidate(
    work: &DiscoveryWork,
    started: Instant,
    attempt_timeout: Duration,
    lexical_candidate: &Path,
    canonical_candidate: &Path,
) -> Result<bool, FfmpegDiscoveryOutcome> {
    let legacy = local_bin_dir().map_err(|error| filesystem_outcome(&error))?;
    if lexical_candidate.starts_with(&legacy) {
        return Ok(true);
    }
    checkpoint(work, started, attempt_timeout).map_err(stop_outcome)?;
    let canonical_legacy = match canonicalize_legacy(&legacy, |path| std::fs::canonicalize(path))? {
        Some(path) => path,
        None => return Ok(false),
    };
    checkpoint(work, started, attempt_timeout).map_err(stop_outcome)?;
    Ok(canonical_candidate.starts_with(canonical_legacy))
}

fn canonicalize_legacy(
    legacy: &Path,
    canonicalize: impl FnOnce(&Path) -> std::io::Result<PathBuf>,
) -> Result<Option<PathBuf>, FfmpegDiscoveryOutcome> {
    match canonicalize(legacy) {
        Ok(path) => Ok(Some(path)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(filesystem_outcome(&error)),
    }
}

fn native_prefix() -> Option<PathBuf> {
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    return Some(PathBuf::from("/opt/homebrew/bin"));
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    return Some(PathBuf::from("/usr/local/bin"));
    #[allow(unreachable_code)]
    None
}

#[cfg(test)]
mod tests;
