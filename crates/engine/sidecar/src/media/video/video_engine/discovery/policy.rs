use std::time::Duration;

/// Centralized RFC 032 discovery bounds shared by Auto and Selected modes.
///
/// `max_raw_path_entries` and `max_path_candidates` are a *reachability*
/// bound, not a performance one (RFC 039): `.take(max_raw_path_entries)` is
/// applied to the raw `PATH` iterator before any entry is inspected, so an
/// entry beyond the cap is never canonicalized, never checked, and never
/// becomes a candidate — its position, not its validity, decides whether
/// discovery ever sees it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FfmpegLocatorPolicy {
    pub max_raw_path_entries: usize,
    pub max_path_candidates: usize,
    pub probe_timeout: Duration,
    pub attempt_timeout: Duration,
}

/// RFC 032's original bounds. Still correct for macOS and Linux: macOS has a
/// cap-exempt native-prefix candidate reserved after this cap
/// ([`super::worker::native_prefix`]), so it is never reachability-blocked by
/// either value here, and no evidence suggests Linux `PATH`s approach these
/// numbers.
const DEFAULT_MAX_RAW_PATH_ENTRIES: usize = 64;
const DEFAULT_MAX_PATH_CANDIDATES: usize = 32;

/// RFC 039: Windows has no cap-exempt fallback — the capped scan is the
/// *only* Auto-mode route there — and both defaults above are undersized
/// against real Windows `PATH`s. Measured on `windows-latest` via RFC 038's
/// native-smoke workflow (2026-08-15, Phase 0): a real inherited `PATH` had
/// **78 raw entries**, of which **66** canonicalized to unique, existing
/// directories — both above the macOS/Linux defaults. Raising only the raw
/// cap without this one would have left `max_path_candidates` as the new,
/// equally-reachable ceiling (confirmed by execution: an early instrumented
/// run with the raw cap raised to 512 and this one left at 32 still returned
/// `SearchLimitReached(CandidateCount)`, exactly RFC 039 §3.2's predicted
/// failure mode).
///
/// 256 / 128 give roughly 3x and 2x headroom over the observed 78 / 66 —
/// a ceiling with room, not a value fitted to one measurement. Timing
/// headroom is not the binding constraint: the same measurement found
/// filesystem collection cost on Windows to be ~80µs per raw entry, so even
/// a full 256-entry scan is on the order of tens of milliseconds against the
/// 6-second `attempt_timeout` — three orders of magnitude of headroom.
/// `attempt_timeout` therefore does not move; see RFC 039 §2.
#[cfg(windows)]
const WINDOWS_MAX_RAW_PATH_ENTRIES: usize = 256;
#[cfg(windows)]
const WINDOWS_MAX_PATH_CANDIDATES: usize = 128;

impl Default for FfmpegLocatorPolicy {
    fn default() -> Self {
        Self {
            #[cfg(windows)]
            max_raw_path_entries: WINDOWS_MAX_RAW_PATH_ENTRIES,
            #[cfg(not(windows))]
            max_raw_path_entries: DEFAULT_MAX_RAW_PATH_ENTRIES,
            #[cfg(windows)]
            max_path_candidates: WINDOWS_MAX_PATH_CANDIDATES,
            #[cfg(not(windows))]
            max_path_candidates: DEFAULT_MAX_PATH_CANDIDATES,
            probe_timeout: Duration::from_secs(2),
            attempt_timeout: Duration::from_secs(6),
        }
    }
}

#[cfg(test)]
mod tests;
