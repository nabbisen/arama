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
