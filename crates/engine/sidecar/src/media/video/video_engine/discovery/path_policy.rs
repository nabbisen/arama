use std::{
    collections::HashSet,
    ffi::OsStr,
    io,
    path::{Path, PathBuf},
};

use super::{DiscoverySource, FfmpegLocatorPolicy, FilesystemIssue, SearchLimit};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NormalizedCandidate {
    pub directory: PathBuf,
    pub source: DiscoverySource,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FilesystemDiagnostic {
    pub issue: FilesystemIssue,
    pub source: DiscoverySource,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CandidatePlanningStop {
    Cancelled,
    SearchLimit(SearchLimit),
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NormalizedPathCandidates {
    pub candidates: Vec<NormalizedCandidate>,
    pub rejected_entries: usize,
    pub raw_truncated: bool,
    pub candidate_truncated: bool,
    pub filesystem_diagnostic: Option<FilesystemDiagnostic>,
    pub stop: Option<CandidatePlanningStop>,
}

pub trait CandidateWorkControl {
    fn checkpoint(&mut self) -> Result<(), CandidatePlanningStop>;
}

impl<F> CandidateWorkControl for F
where
    F: FnMut() -> Result<(), CandidatePlanningStop>,
{
    fn checkpoint(&mut self) -> Result<(), CandidatePlanningStop> {
        self()
    }
}

/// Filesystem operations are deliberately separated so the normalizer can
/// check cancellation and the monotonic attempt budget around every call.
pub trait CandidateFilesystem {
    fn canonicalize(&mut self, path: &Path) -> io::Result<PathBuf>;
    fn is_directory(&mut self, path: &Path) -> io::Result<bool>;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct StdCandidateFilesystem;

impl CandidateFilesystem for StdCandidateFilesystem {
    fn canonicalize(&mut self, path: &Path) -> io::Result<PathBuf> {
        std::fs::canonicalize(path)
    }

    fn is_directory(&mut self, path: &Path) -> io::Result<bool> {
        Ok(std::fs::metadata(path)?.is_dir())
    }
}

pub fn normalize_auto_candidates<C, F>(
    path: Option<&OsStr>,
    native_prefix: Option<&Path>,
    policy: FfmpegLocatorPolicy,
    control: &mut C,
    filesystem: &mut F,
) -> NormalizedPathCandidates
where
    C: CandidateWorkControl,
    F: CandidateFilesystem,
{
    let mut result = NormalizedPathCandidates::default();
    let mut identities = HashSet::new();

    if let Some(path) = path {
        let mut entries = std::env::split_paths(path);
        for entry in entries.by_ref().take(policy.max_raw_path_entries) {
            if entry.as_os_str().is_empty() || !entry.is_absolute() {
                result.rejected_entries += 1;
                continue;
            }
            let Some(canonical) = checked_directory_identity(
                &entry,
                DiscoverySource::AutoPath,
                control,
                filesystem,
                &mut result,
            ) else {
                if result.stop.is_some() {
                    return result;
                }
                continue;
            };
            let identity = logical_identity(&canonical);
            if identities.contains(&identity) {
                continue;
            }
            if result.candidates.len() == policy.max_path_candidates {
                result.candidate_truncated = true;
                continue;
            }
            identities.insert(identity);
            result.candidates.push(NormalizedCandidate {
                directory: canonical,
                source: DiscoverySource::AutoPath,
            });
        }
        result.raw_truncated = entries.next().is_some();
    }

    if let Some(native_prefix) = native_prefix {
        let Some(canonical) = checked_directory_identity(
            native_prefix,
            DiscoverySource::NativePrefix,
            control,
            filesystem,
            &mut result,
        ) else {
            return result;
        };
        let identity = logical_identity(&canonical);
        if identities.insert(identity) {
            result.candidates.push(NormalizedCandidate {
                directory: canonical,
                source: DiscoverySource::NativePrefix,
            });
        }
    }

    result
}

fn checked_directory_identity<C, F>(
    path: &Path,
    source: DiscoverySource,
    control: &mut C,
    filesystem: &mut F,
    result: &mut NormalizedPathCandidates,
) -> Option<PathBuf>
where
    C: CandidateWorkControl,
    F: CandidateFilesystem,
{
    if checkpoint(control, result).is_err() {
        return None;
    }
    let canonical = match filesystem.canonicalize(path) {
        Ok(canonical) => canonical,
        Err(error) => {
            record_missing_or_filesystem_error(result, source, &error);
            let _ = checkpoint(control, result);
            return None;
        }
    };
    if checkpoint(control, result).is_err() {
        return None;
    }

    let is_directory = match filesystem.is_directory(&canonical) {
        Ok(is_directory) => is_directory,
        Err(error) => {
            record_missing_or_filesystem_error(result, source, &error);
            let _ = checkpoint(control, result);
            return None;
        }
    };
    if checkpoint(control, result).is_err() {
        return None;
    }
    if !is_directory {
        result.rejected_entries += 1;
        return None;
    }
    Some(canonical)
}

fn checkpoint<C: CandidateWorkControl>(
    control: &mut C,
    result: &mut NormalizedPathCandidates,
) -> Result<(), ()> {
    control.checkpoint().map_err(|stop| {
        result.stop = Some(stop);
    })
}

/// Task 019: a `PATH` entry that simply does not exist (`NotFound`) is
/// ordinary `PATH` hygiene — a stale directory left behind by an
/// uninstalled tool, a per-user directory never created — not a filesystem
/// problem. Counting it as a plain rejected entry, the same bucket
/// `!is_directory` and empty/relative entries already use, keeps `Missing`
/// reachable when nothing else is wrong: `run_discovery_work`
/// (`worker.rs`) seeds its failure outcome from `filesystem_diagnostic`
/// when one is set and only falls back to `Missing` when it is `None`, and
/// a genuinely dangling `PATH` is common enough on real Windows machines
/// (8 of 78 raw entries on `windows-latest`, RFC 039's Phase 0 measurement)
/// that treating it as a diagnostic-worthy failure made the one message
/// that tells a user to install ffmpeg effectively unreachable in the
/// common case. A real access or identity problem — `PermissionDenied`, or
/// anything else `NotFound` does not already cover — still becomes a
/// diagnostic exactly as before.
fn record_missing_or_filesystem_error(
    result: &mut NormalizedPathCandidates,
    source: DiscoverySource,
    error: &io::Error,
) {
    if error.kind() == io::ErrorKind::NotFound {
        result.rejected_entries += 1;
        return;
    }
    record_filesystem_error(result, source, error);
}

fn record_filesystem_error(
    result: &mut NormalizedPathCandidates,
    source: DiscoverySource,
    error: &io::Error,
) {
    let issue = if error.kind() == io::ErrorKind::PermissionDenied {
        FilesystemIssue::Access
    } else {
        FilesystemIssue::MetadataOrIdentity
    };
    let replace = match result.filesystem_diagnostic {
        None => true,
        Some(existing) => {
            filesystem_issue_priority(issue) > filesystem_issue_priority(existing.issue)
        }
    };
    if replace {
        result.filesystem_diagnostic = Some(FilesystemDiagnostic { issue, source });
    }
}

fn filesystem_issue_priority(issue: FilesystemIssue) -> u8 {
    match issue {
        FilesystemIssue::Access => 1,
        FilesystemIssue::MetadataOrIdentity => 0,
    }
}

#[cfg(not(windows))]
fn logical_identity(path: &Path) -> PathBuf {
    path.to_path_buf()
}

#[cfg(windows)]
fn logical_identity(path: &Path) -> PathBuf {
    let mut value = path.to_string_lossy().replace('/', "\\");
    if let Some(stripped) = value.strip_prefix(r"\\?\UNC\") {
        value = format!(r"\\{stripped}");
    } else if let Some(stripped) = value.strip_prefix(r"\\?\") {
        value = stripped.to_owned();
    }
    PathBuf::from(value.to_lowercase())
}

#[cfg(test)]
mod tests;
