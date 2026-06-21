use crate::pipeline::score::similarity::video::util::{deduplicate_by_gap, uniform_timestamps};

use super::*;

fn cfg() -> VideoSimilarityConfig {
    VideoSimilarityConfig::default()
}

// Default config layout (for reference):
//   head_fixed_anchors_secs : [3.0, 9.0, 15.0]   — always kept, gap-exempt
//   head_zone_secs          : 135.0
//   head_sample_count       : 5
//   percent_anchors         : [0.30, 0.50, 0.70]
//   tail_anchors_secs       : [30.0, 15.0, 5.0]
//   min_sample_gap_secs     : 20.0

#[test]
fn test_1hour() {
    let ts = cfg().compute_sample_timestamps(3600.0);

    // Fixed anchors are always present regardless of gap.
    let fixed = [3.0_f64, 9.0, 15.0];
    for &f in &fixed {
        assert!(ts.contains(&f), "fixed anchor {f} missing: {ts:?}");
    }

    // All timestamps are within bounds.
    assert!(ts[0] > 0.0);
    assert!(*ts.last().unwrap() < 3600.0);

    // Non-fixed consecutive pairs must respect the minimum gap.
    for w in ts.windows(2) {
        let both_free = fixed.iter().all(|&f| (f - w[0]).abs() > 1e-9)
            && fixed.iter().all(|&f| (f - w[1]).abs() > 1e-9);
        if both_free {
            assert!(
                w[1] - w[0] >= 20.0,
                "gap too small between non-fixed points: {:?}",
                w
            );
        }
    }

    // Expected point count from the default config:
    //   3 fixed head anchors + 5 uniform head points + 3 pct + 3 tail = 14
    //   After dedup with gap=20 (fixed exempt):
    //   head uniform: step = (135-15)/6 = 20 → [35, 55, 75, 95, 115] — all kept
    //   pct: 1080, 1800, 2520 — all >20s from previous
    //   tail: 3570, 3585, 3595 — 3585-3570=15 < 20, dropped; 3595-3570=25 kept
    //   total = 3 + 5 + 3 + 2 = 13
    assert_eq!(ts.len(), 13, "unexpected count: {ts:?}");
}

#[test]
fn test_90s() {
    let ts = cfg().compute_sample_timestamps(90.0);

    // All timestamps are within bounds.
    assert!(ts.iter().all(|&t| t > 0.0 && t < 90.0), "{ts:?}");

    // Fixed anchors are present.
    let fixed = [3.0_f64, 9.0, 15.0];
    for &f in &fixed {
        assert!(ts.contains(&f), "fixed anchor {f} missing: {ts:?}");
    }

    // Non-fixed consecutive pairs must respect the minimum gap.
    for w in ts.windows(2) {
        let both_free = fixed.iter().all(|&f| (f - w[0]).abs() > 1e-9)
            && fixed.iter().all(|&f| (f - w[1]).abs() > 1e-9);
        if both_free {
            assert!(
                w[1] - w[0] >= 20.0,
                "gap too small between non-fixed points: {:?}",
                w
            );
        }
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
