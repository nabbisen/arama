use std::{io, path::Path};

use super::{canonicalize_legacy, incomplete_pair_outcome};
use crate::media::video::video_engine::discovery::{
    FfmpegDiscoveryOutcome, FilesystemIssue, PairIssue,
};

#[test]
fn zero_one_and_two_executables_have_distinct_pair_classification() {
    assert_eq!(
        incomplete_pair_outcome([false, false]),
        Some(FfmpegDiscoveryOutcome::Missing)
    );
    assert_eq!(
        incomplete_pair_outcome([true, false]),
        Some(FfmpegDiscoveryOutcome::InvalidPair(
            PairIssue::MissingMember
        ))
    );
    assert_eq!(
        incomplete_pair_outcome([false, true]),
        Some(FfmpegDiscoveryOutcome::InvalidPair(
            PairIssue::MissingMember
        ))
    );
    assert_eq!(incomplete_pair_outcome([true, true]), None);
}

#[test]
fn absent_legacy_root_is_the_only_ignored_identity_failure() {
    let legacy = Path::new("/legacy");
    assert_eq!(
        canonicalize_legacy(legacy, |_| Err(io::Error::from(io::ErrorKind::NotFound))),
        Ok(None)
    );
    assert_eq!(
        canonicalize_legacy(legacy, |_| {
            Err(io::Error::from(io::ErrorKind::PermissionDenied))
        }),
        Err(FfmpegDiscoveryOutcome::FilesystemUnavailable(
            FilesystemIssue::Access
        ))
    );
    assert_eq!(
        canonicalize_legacy(legacy, |_| Err(io::Error::from(io::ErrorKind::InvalidData))),
        Err(FfmpegDiscoveryOutcome::FilesystemUnavailable(
            FilesystemIssue::MetadataOrIdentity
        ))
    );
}
