use std::time::Duration;

use super::FfmpegLocatorPolicy;

#[test]
fn production_bounds_are_centralized() {
    let policy = FfmpegLocatorPolicy::default();
    #[cfg(windows)]
    {
        assert_eq!(policy.max_raw_path_entries, 256);
        assert_eq!(policy.max_path_candidates, 128);
    }
    #[cfg(not(windows))]
    {
        assert_eq!(policy.max_raw_path_entries, 64);
        assert_eq!(policy.max_path_candidates, 32);
    }
    assert_eq!(policy.probe_timeout, Duration::from_secs(2));
    assert_eq!(policy.attempt_timeout, Duration::from_secs(6));
}

/// RFC 039: macOS and Linux keep RFC 032's original bounds unconditionally —
/// this is the asymmetry that makes the platform-conditional default a
/// deliberate choice rather than an oversight for the two platforms that
/// were not remeasured.
#[test]
#[cfg(not(windows))]
fn non_windows_bounds_are_unchanged_from_rfc_032() {
    let policy = FfmpegLocatorPolicy::default();
    assert_eq!(policy.max_raw_path_entries, 64);
    assert_eq!(policy.max_path_candidates, 32);
}
