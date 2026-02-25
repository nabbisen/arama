// todo: verify tests run
use crate::pipeline::score::similarity::video::util::{deduplicate_by_gap, uniform_timestamps};

use super::*;

fn cfg() -> VideoSimilarityConfig {
    VideoSimilarityConfig::default()
}

#[test]
fn test_1hour() {
    let ts = cfg().compute_sample_timestamps(3600.0);
    assert_eq!(ts.len(), 12);
    assert!(ts[0] > 0.0);
    assert!(*ts.last().unwrap() < 3600.0);
    for w in ts.windows(2) {
        assert!(w[1] - w[0] >= 20.0, "gap too small: {:?}", w);
    }
}

#[test]
fn test_90s() {
    let ts = cfg().compute_sample_timestamps(90.0);
    assert!(ts.iter().all(|&t| t > 0.0 && t < 90.0));
    for w in ts.windows(2) {
        assert!(w[1] - w[0] >= 20.0);
    }
}

#[test]
fn test_uniform() {
    let ts = uniform_timestamps(6, 120.0);
    assert_eq!(ts.len(), 6);
    assert!((ts[0] - 120.0 / 7.0).abs() < 0.01);
}

#[test]
fn test_dedup() {
    let input = vec![5.0, 10.0, 30.0, 35.0, 60.0];
    assert_eq!(deduplicate_by_gap(input, 20.0), vec![5.0, 30.0, 60.0]);
}
