use std::time::Duration;

use super::FfmpegLocatorPolicy;

#[test]
fn production_bounds_are_centralized() {
    let policy = FfmpegLocatorPolicy::default();
    assert_eq!(policy.max_raw_path_entries, 64);
    assert_eq!(policy.max_path_candidates, 32);
    assert_eq!(policy.probe_timeout, Duration::from_secs(2));
    assert_eq!(policy.attempt_timeout, Duration::from_secs(6));
}
