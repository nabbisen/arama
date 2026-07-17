mod coordinator;
mod outcome;
mod path_policy;
mod policy;
mod preference;
mod runtime;
mod worker;

pub use coordinator::{
    CoordinatorPublication, DiscoveryRequestDisposition, DiscoveryWork, FfmpegDiscoveryCoordinator,
};
pub use outcome::{
    DiscoverySource, FfmpegDiscoveryFailure, FfmpegDiscoveryOutcome, FilesystemIssue, PairIssue,
    SearchLimit,
};
pub use path_policy::{
    CandidateFilesystem, CandidatePlanningStop, CandidateWorkControl, FilesystemDiagnostic,
    NormalizedCandidate, NormalizedPathCandidates, StdCandidateFilesystem,
    normalize_auto_candidates,
};
pub use policy::FfmpegLocatorPolicy;
pub use preference::{
    PreferenceRetainReason, PreferenceTransition, SelectionCandidate, ValidatedSelection,
    clear_selection, prepare_selection, publish_validated_selection, reject_selection,
};
pub use runtime::{FfmpegDiscoveryEvent, FfmpegDiscoveryRuntime, FfmpegDiscoveryTicket};
