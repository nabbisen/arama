use std::{
    io,
    path::Path,
    sync::{Arc, Mutex, atomic::AtomicBool},
    time::{Duration, Instant},
};

use arama_env::ffmpeg_location::FfmpegLocationPreference;

use super::{canonicalize_legacy, incomplete_pair_outcome, is_legacy_candidate};
use crate::media::video::video_engine::discovery::{
    DiscoveryWork, FfmpegDiscoveryOutcome, FilesystemIssue, PairIssue,
};

/// Same defect class as Task 023's `ARAMA_DATA_HOME` race
/// (`env/src/dir.rs`): `local_bin_dir()`/`legacy_local_dir()` read a
/// process-global env var, and Rust runs tests in parallel by default. One
/// test here mutates `ARAMA_DATA_HOME`; another reads its real, ambient
/// value via the same functions - without serialising, a candidate
/// computed in one test's critical section could be checked against a
/// `local_bin_dir()` resolved under the *other* test's override. The pure
/// -seam fix Task 023 used isn't available across this crate boundary
/// (`arama_env`'s override-injection seam is private to that crate), so -
/// deliberately, per that task's own precedent for exactly this shape -
/// this is the second-preference route: serialise instead.
static ARAMA_DATA_HOME_TEST_LOCK: Mutex<()> = Mutex::new(());

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

/// Task 029: `local_bin_dir()` (arama's *current* data-directory `bin/`)
/// and `legacy_local_dir()` (the true, always-exe-relative pre-0.40.0
/// managed-ffmpeg location) diverge whenever `ARAMA_DATA_HOME` is set -
/// exactly the seam RFC 041 built for test isolation, reused here instead
/// of mutating real platform directories. Before the fix, `is_legacy_candidate`
/// followed only `local_bin_dir()`, so a candidate sitting at the *true*
/// legacy location was not recognised as legacy the moment `local_dir()`
/// pointed anywhere else - which is unconditionally true post-RFC-041,
/// `ARAMA_DATA_HOME` or not. This pins the guard to the location RFC 032
/// actually specifies.
#[test]
fn is_legacy_candidate_flags_the_true_legacy_bin_dir_regardless_of_local_dir() {
    let _guard = ARAMA_DATA_HOME_TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let previous = std::env::var_os(arama_env::DATA_HOME_ENV_VAR);
    let scratch = std::env::temp_dir().join(format!(
        "arama-legacy-guard-test-{}-unrelated-data-home",
        std::process::id()
    ));
    unsafe {
        std::env::set_var(arama_env::DATA_HOME_ENV_VAR, &scratch);
    }

    let true_legacy_bin = arama_env::legacy_local_dir().unwrap().join("bin");
    let work = DiscoveryWork {
        generation: 0,
        preference: FfmpegLocationPreference::Auto,
        cancellation: Arc::new(AtomicBool::new(false)),
    };
    let result = is_legacy_candidate(
        &work,
        Instant::now(),
        Duration::from_secs(5),
        &true_legacy_bin,
        &true_legacy_bin,
    );

    unsafe {
        match &previous {
            Some(value) => std::env::set_var(arama_env::DATA_HOME_ENV_VAR, value),
            None => std::env::remove_var(arama_env::DATA_HOME_ENV_VAR),
        }
    }

    assert_eq!(
        result,
        Ok(true),
        "the true, exe-relative legacy ffmpeg bin/ directory must be excluded \
         from automatic discovery regardless of where ARAMA_DATA_HOME (or \
         anything else) currently points local_dir() at"
    );
}

/// The other half of Task 029's fix: arama's own *current* data-directory
/// `bin/` must also stay excluded, on the same "never trust for automatic
/// discovery" reasoning - independent of the legacy-location fix above.
#[test]
fn is_legacy_candidate_also_flags_the_current_data_directory_bin_dir() {
    let _guard = ARAMA_DATA_HOME_TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let candidate = arama_env::local_bin_dir().unwrap();
    let work = DiscoveryWork {
        generation: 0,
        preference: FfmpegLocationPreference::Auto,
        cancellation: Arc::new(AtomicBool::new(false)),
    };

    let result = is_legacy_candidate(
        &work,
        Instant::now(),
        Duration::from_secs(5),
        &candidate,
        &candidate,
    );

    assert_eq!(result, Ok(true));
}
