use std::{
    cell::Cell,
    collections::HashMap,
    ffi::OsString,
    fs, io,
    path::{Path, PathBuf},
    rc::Rc,
};

use super::{
    CandidateFilesystem, CandidatePlanningStop, FilesystemDiagnostic, NormalizedCandidate,
    StdCandidateFilesystem, normalize_auto_candidates,
};
use crate::media::video::video_engine::discovery::{
    DiscoverySource, FfmpegLocatorPolicy, FilesystemIssue, SearchLimit,
};

fn root(name: &str) -> PathBuf {
    std::env::current_dir()
        .unwrap()
        .join("target/discovery-policy-tests")
        .join(format!("{name}-{}", std::process::id()))
}

fn never_stop() -> impl FnMut() -> Result<(), CandidatePlanningStop> {
    || Ok(())
}

#[test]
fn empty_relative_duplicate_and_limits_are_deterministic() {
    let root = root("normalize");
    let first = root.join("first");
    let second = root.join("second");
    fs::create_dir_all(&first).unwrap();
    fs::create_dir_all(&second).unwrap();
    let path = std::env::join_paths([
        first.clone(),
        first.clone(),
        second.clone(),
        PathBuf::new(),
        PathBuf::from("relative"),
    ])
    .unwrap();
    let policy = FfmpegLocatorPolicy {
        max_raw_path_entries: 5,
        max_path_candidates: 1,
        ..FfmpegLocatorPolicy::default()
    };
    let mut control = never_stop();
    let mut filesystem = StdCandidateFilesystem;

    let result =
        normalize_auto_candidates(Some(&path), None, policy, &mut control, &mut filesystem);

    assert_eq!(
        result.candidates,
        vec![NormalizedCandidate {
            directory: fs::canonicalize(first).unwrap(),
            source: DiscoverySource::AutoPath,
        }]
    );
    assert_eq!(result.rejected_entries, 2);
    assert!(result.candidate_truncated);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn raw_entry_limit_and_reserved_native_candidate_are_reported() {
    let root = root("native-reserved");
    let first = root.join("first");
    let second = root.join("second");
    let native = root.join("native");
    for directory in [&first, &second, &native] {
        fs::create_dir_all(directory).unwrap();
    }
    let path: OsString = std::env::join_paths([first.clone(), second]).unwrap();
    let policy = FfmpegLocatorPolicy {
        max_raw_path_entries: 1,
        max_path_candidates: 1,
        ..FfmpegLocatorPolicy::default()
    };
    let mut control = never_stop();
    let mut filesystem = StdCandidateFilesystem;

    let result = normalize_auto_candidates(
        Some(&path),
        Some(&native),
        policy,
        &mut control,
        &mut filesystem,
    );

    assert_eq!(result.candidates.len(), 2);
    assert_eq!(result.candidates[1].source, DiscoverySource::NativePrefix);
    assert!(result.raw_truncated);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn native_prefix_after_saturated_path_keeps_its_reserved_slot() {
    let root = root("native-after-capacity");
    let native = root.join("native");
    fs::create_dir_all(&native).unwrap();
    let mut path_entries = Vec::new();
    for index in 0..32 {
        let directory = root.join(format!("path-{index}"));
        fs::create_dir_all(&directory).unwrap();
        path_entries.push(directory);
    }
    path_entries.push(native.clone());
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;

        let alias = root.join("native-alias");
        symlink(&native, &alias).unwrap();
        path_entries.push(alias);
    }
    let path = std::env::join_paths(path_entries).unwrap();
    let mut control = never_stop();
    let mut filesystem = StdCandidateFilesystem;

    let result = normalize_auto_candidates(
        Some(&path),
        Some(&native),
        FfmpegLocatorPolicy::default(),
        &mut control,
        &mut filesystem,
    );

    assert_eq!(result.candidates.len(), 33);
    assert_eq!(
        result.candidates.last().unwrap(),
        &NormalizedCandidate {
            directory: fs::canonicalize(&native).unwrap(),
            source: DiscoverySource::NativePrefix,
        }
    );
    assert!(result.candidate_truncated);
    fs::remove_dir_all(root).unwrap();
}

/// RFC 039's regression guard: nothing in the suite exercised the raw cap at
/// real scale before this (Task 018's finding) — the raw-entry mechanism was
/// tested only via a synthetic `max_raw_path_entries: 1` override, and the
/// candidate-cap mechanism only with well under 64 raw entries. This
/// constructs a `PATH` at the scale RFC 039's Phase 0 measurement actually
/// observed on `windows-latest` (78 raw entries, 66 unique candidates) and
/// proves the mechanism, not just the number: a valid, unique directory
/// beyond the old 64-entry cap is unreachable at the old default and
/// reachable at the raised one — using the exact policy values
/// `FfmpegLocatorPolicy::default()` now returns on Windows, not a rounder
/// stand-in, so this test would fail if those values and this one drifted
/// apart.
#[test]
fn valid_directory_beyond_old_cap_is_reachable_only_at_raised_scale() {
    let root = root("real-scale-windows-path");
    let mut path_entries = Vec::new();
    for index in 0..90 {
        let directory = root.join(format!("tool-{index}"));
        fs::create_dir_all(&directory).unwrap();
        path_entries.push(directory);
    }
    // Position 70: past the old 64-entry cap, well inside the raised 256.
    let target = root.join("tool-70");
    let path = std::env::join_paths(&path_entries).unwrap();

    let old_default = FfmpegLocatorPolicy {
        max_raw_path_entries: 64,
        max_path_candidates: 32,
        ..FfmpegLocatorPolicy::default()
    };
    let raised = FfmpegLocatorPolicy {
        max_raw_path_entries: 256,
        max_path_candidates: 128,
        ..FfmpegLocatorPolicy::default()
    };

    let mut control = never_stop();
    let mut filesystem = StdCandidateFilesystem;
    let at_old_default = normalize_auto_candidates(
        Some(&path),
        None,
        old_default,
        &mut control,
        &mut filesystem,
    );
    let target_canonical = fs::canonicalize(&target).unwrap();
    assert!(
        !at_old_default
            .candidates
            .iter()
            .any(|candidate| candidate.directory == target_canonical),
        "test setup error: the old cap should not reach position 70"
    );
    assert!(at_old_default.raw_truncated);

    let mut control = never_stop();
    let at_raised_scale =
        normalize_auto_candidates(Some(&path), None, raised, &mut control, &mut filesystem);
    assert!(
        at_raised_scale
            .candidates
            .iter()
            .any(|candidate| candidate.directory == target_canonical),
        "a valid directory at raw position 70 must be reachable once the cap is raised past it"
    );
    assert!(!at_raised_scale.raw_truncated);
    assert!(!at_raised_scale.candidate_truncated);

    fs::remove_dir_all(root).unwrap();
}

struct CountingFilesystem {
    operations: Rc<Cell<usize>>,
    identities: HashMap<PathBuf, io::Result<PathBuf>>,
}

impl CandidateFilesystem for CountingFilesystem {
    fn canonicalize(&mut self, path: &Path) -> io::Result<PathBuf> {
        self.operations.set(self.operations.get() + 1);
        self.identities
            .remove(path)
            .unwrap_or_else(|| Ok(path.to_path_buf()))
    }

    fn is_directory(&mut self, _path: &Path) -> io::Result<bool> {
        self.operations.set(self.operations.get() + 1);
        Ok(true)
    }
}

#[test]
fn cancellation_after_one_canonicalization_starts_no_second_filesystem_operation() {
    let operations = Rc::new(Cell::new(0));
    let observed = operations.clone();
    let mut filesystem = CountingFilesystem {
        operations,
        identities: HashMap::new(),
    };
    let path = std::env::join_paths([absolute("first"), absolute("second")]).unwrap();
    let mut control = move || {
        if observed.get() == 1 {
            Err(CandidatePlanningStop::Cancelled)
        } else {
            Ok(())
        }
    };

    let result = normalize_auto_candidates(
        Some(&path),
        None,
        FfmpegLocatorPolicy::default(),
        &mut control,
        &mut filesystem,
    );

    assert_eq!(filesystem.operations.get(), 1);
    assert_eq!(result.stop, Some(CandidatePlanningStop::Cancelled));
}

#[test]
fn budget_expiry_between_candidates_starts_no_next_identity_operation() {
    let operations = Rc::new(Cell::new(0));
    let observed = operations.clone();
    let mut filesystem = CountingFilesystem {
        operations,
        identities: HashMap::new(),
    };
    let path = std::env::join_paths([absolute("first"), absolute("second")]).unwrap();
    let mut control = move || {
        if observed.get() == 2 {
            Err(CandidatePlanningStop::SearchLimit(
                SearchLimit::WholeAttempt,
            ))
        } else {
            Ok(())
        }
    };

    let result = normalize_auto_candidates(
        Some(&path),
        None,
        FfmpegLocatorPolicy::default(),
        &mut control,
        &mut filesystem,
    );

    assert_eq!(filesystem.operations.get(), 2);
    assert_eq!(
        result.stop,
        Some(CandidatePlanningStop::SearchLimit(
            SearchLimit::WholeAttempt
        ))
    );
}

#[test]
fn native_prefix_obeys_cancellation_before_its_filesystem_work() {
    let operations = Rc::new(Cell::new(0));
    let observed = operations.clone();
    let mut filesystem = CountingFilesystem {
        operations,
        identities: HashMap::new(),
    };
    let path = std::env::join_paths([absolute("path")]).unwrap();
    let mut checkpoints = 0;
    let mut control = move || {
        checkpoints += 1;
        if observed.get() == 2 && checkpoints >= 4 {
            Err(CandidatePlanningStop::Cancelled)
        } else {
            Ok(())
        }
    };

    let result = normalize_auto_candidates(
        Some(&path),
        Some(&absolute("native")),
        FfmpegLocatorPolicy::default(),
        &mut control,
        &mut filesystem,
    );

    assert_eq!(filesystem.operations.get(), 2);
    assert_eq!(result.stop, Some(CandidatePlanningStop::Cancelled));
}

#[test]
fn access_diagnostic_retains_issue_and_candidate_source() {
    let denied = absolute("denied");
    let missing = absolute("missing");
    let mut identities = HashMap::new();
    identities.insert(
        denied.clone(),
        Err(io::Error::new(io::ErrorKind::PermissionDenied, "denied")),
    );
    identities.insert(
        missing.clone(),
        Err(io::Error::new(io::ErrorKind::NotFound, "missing")),
    );
    let mut filesystem = CountingFilesystem {
        operations: Rc::new(Cell::new(0)),
        identities,
    };
    let path = std::env::join_paths([denied, missing]).unwrap();
    let mut control = never_stop();

    let result = normalize_auto_candidates(
        Some(&path),
        None,
        FfmpegLocatorPolicy::default(),
        &mut control,
        &mut filesystem,
    );

    assert_eq!(
        result.filesystem_diagnostic,
        Some(FilesystemDiagnostic {
            issue: FilesystemIssue::Access,
            source: DiscoverySource::AutoPath,
        })
    );
}

/// Task 019: a `PATH` made entirely of dangling entries - the ordinary case
/// on a real Windows machine (RFC 039's Phase 0 measurement found 8 of 78 on
/// `windows-latest`) - must not produce a filesystem diagnostic. Before this
/// fix, every `NotFound` was recorded exactly like a genuine access/identity
/// failure, which made `run_discovery_work`'s `Missing` fallback (`worker.rs`)
/// unreachable in the single most common real-world "ffmpeg is not
/// installed" case on Windows.
#[test]
fn dangling_entries_alone_produce_no_filesystem_diagnostic() {
    let first = absolute("dangling-first");
    let second = absolute("dangling-second");
    let mut identities = HashMap::new();
    identities.insert(
        first.clone(),
        Err(io::Error::new(io::ErrorKind::NotFound, "first")),
    );
    identities.insert(
        second.clone(),
        Err(io::Error::new(io::ErrorKind::NotFound, "second")),
    );
    let mut filesystem = CountingFilesystem {
        operations: Rc::new(Cell::new(0)),
        identities,
    };
    let path = std::env::join_paths([first, second]).unwrap();
    let mut control = never_stop();

    let result = normalize_auto_candidates(
        Some(&path),
        None,
        FfmpegLocatorPolicy::default(),
        &mut control,
        &mut filesystem,
    );

    assert_eq!(result.filesystem_diagnostic, None);
    assert_eq!(result.rejected_entries, 2);
    assert!(result.candidates.is_empty());
}

/// A genuine access/identity problem must still be recorded even when
/// dangling entries are also present, so real diagnostics are not
/// collateral damage of the Task 019 fix.
#[test]
fn a_real_filesystem_error_is_still_recorded_alongside_dangling_entries() {
    let dangling = absolute("dangling");
    let denied = absolute("denied");
    let mut identities = HashMap::new();
    identities.insert(
        dangling.clone(),
        Err(io::Error::new(io::ErrorKind::NotFound, "dangling")),
    );
    identities.insert(
        denied.clone(),
        Err(io::Error::new(io::ErrorKind::PermissionDenied, "denied")),
    );
    let mut filesystem = CountingFilesystem {
        operations: Rc::new(Cell::new(0)),
        identities,
    };
    let path = std::env::join_paths([dangling, denied]).unwrap();
    let mut control = never_stop();

    let result = normalize_auto_candidates(
        Some(&path),
        None,
        FfmpegLocatorPolicy::default(),
        &mut control,
        &mut filesystem,
    );

    assert_eq!(result.rejected_entries, 1);
    assert_eq!(
        result.filesystem_diagnostic,
        Some(FilesystemDiagnostic {
            issue: FilesystemIssue::Access,
            source: DiscoverySource::AutoPath,
        })
    );
}

#[cfg(unix)]
#[test]
fn symlink_alias_is_normalized_to_one_logical_candidate() {
    use std::os::unix::fs::symlink;

    let root = root("symlink-alias");
    let real = root.join("real");
    let alias = root.join("alias");
    fs::create_dir_all(&real).unwrap();
    symlink(&real, &alias).unwrap();
    let path = std::env::join_paths([real, alias]).unwrap();
    let mut control = never_stop();
    let mut filesystem = StdCandidateFilesystem;

    let result = normalize_auto_candidates(
        Some(&path),
        None,
        FfmpegLocatorPolicy::default(),
        &mut control,
        &mut filesystem,
    );

    assert_eq!(result.candidates.len(), 1);
    fs::remove_dir_all(root).unwrap();
}

fn absolute(name: &str) -> PathBuf {
    if cfg!(windows) {
        PathBuf::from(format!(r"C:\{name}"))
    } else {
        PathBuf::from(format!("/{name}"))
    }
}
