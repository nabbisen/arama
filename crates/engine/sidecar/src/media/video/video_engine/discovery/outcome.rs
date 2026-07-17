use super::super::FfmpegToolchain;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiscoverySource {
    AutoPath,
    NativePrefix,
    SelectedDirectory,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PairIssue {
    MissingMember,
    MalformedVersion,
    VersionMismatch,
    OutputLimit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SearchLimit {
    CandidateCount,
    WholeAttempt,
    WorkerDraining,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FilesystemIssue {
    Access,
    MetadataOrIdentity,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FfmpegDiscoveryOutcome {
    Ready {
        toolchain: FfmpegToolchain,
        source: DiscoverySource,
    },
    Missing,
    InvalidPair(PairIssue),
    ProbeTimedOut,
    SearchLimitReached(SearchLimit),
    LegacyLocationExcluded,
    InvalidSearchPath,
    FilesystemUnavailable(FilesystemIssue),
}

/// A discovery result that is guaranteed not to carry Ready authority.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FfmpegDiscoveryFailure(FfmpegDiscoveryOutcome);

impl FfmpegDiscoveryFailure {
    pub fn into_outcome(self) -> FfmpegDiscoveryOutcome {
        self.0
    }
}

impl TryFrom<FfmpegDiscoveryOutcome> for FfmpegDiscoveryFailure {
    type Error = FfmpegDiscoveryOutcome;

    fn try_from(outcome: FfmpegDiscoveryOutcome) -> Result<Self, Self::Error> {
        if matches!(outcome, FfmpegDiscoveryOutcome::Ready { .. }) {
            Err(outcome)
        } else {
            Ok(Self(outcome))
        }
    }
}

impl FfmpegDiscoveryOutcome {
    pub const fn diagnostic_priority(&self) -> u8 {
        match self {
            Self::Ready { .. } => u8::MAX,
            Self::SearchLimitReached(_) => 6,
            Self::ProbeTimedOut => 5,
            Self::FilesystemUnavailable(_) => 4,
            Self::InvalidPair(_) => 3,
            Self::InvalidSearchPath => 2,
            Self::LegacyLocationExcluded => 1,
            Self::Missing => 0,
        }
    }

    pub fn prefer(self, other: Self) -> Self {
        if other.diagnostic_priority() > self.diagnostic_priority() {
            other
        } else {
            self
        }
    }
}

#[cfg(test)]
mod tests;
