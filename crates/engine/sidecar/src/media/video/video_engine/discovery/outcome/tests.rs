use super::{FfmpegDiscoveryOutcome, FilesystemIssue, PairIssue, SearchLimit};

#[test]
fn auto_diagnostics_follow_rfc_precedence() {
    let outcome = FfmpegDiscoveryOutcome::Missing
        .prefer(FfmpegDiscoveryOutcome::LegacyLocationExcluded)
        .prefer(FfmpegDiscoveryOutcome::InvalidSearchPath)
        .prefer(FfmpegDiscoveryOutcome::InvalidPair(
            PairIssue::VersionMismatch,
        ))
        .prefer(FfmpegDiscoveryOutcome::FilesystemUnavailable(
            FilesystemIssue::Access,
        ))
        .prefer(FfmpegDiscoveryOutcome::ProbeTimedOut)
        .prefer(FfmpegDiscoveryOutcome::SearchLimitReached(
            SearchLimit::CandidateCount,
        ));
    assert_eq!(
        outcome,
        FfmpegDiscoveryOutcome::SearchLimitReached(SearchLimit::CandidateCount)
    );
}
