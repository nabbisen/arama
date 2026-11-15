use std::cmp::Ordering;

use rayon::{
    iter::{IntoParallelIterator, ParallelIterator},
    slice::ParallelSliceMut,
};

use super::scan_control::ScanControl;

// Assumption: vectors were L2-normalized when cached.
// Return: left (path, thumbnail_path), right (path, thumbnail_path), similarity.
pub type MediaRef = (String, Option<String>);
pub type SimilarImagePair = (MediaRef, MediaRef, f32);

/// All image pairs at or above `threshold`, globally sorted
/// (highest-similarity first). Deliberately uncapped, matching
/// `find_similar_video_pairs` - RFC 047 S3.3 moved truncation to the
/// caller so a *single* cap applies to the merged image-and-video list,
/// not one cap per media type (images used to be capped here at a fixed
/// 50 while videos had no cap at all, an asymmetry nobody chose). The
/// caller needs the honest pre-cap total to render retained-versus-total
/// counts, which an internal cap would have made impossible to recover.
///
/// `control` is checked once per outer row, before that row's O(n) inner
/// loop - review 161 #2: once cancelled, this turns the remaining work
/// from O(n) per row into O(1) per row, rather than merely stopping
/// *future* calls into this function the way an `.abortable()` `Task`
/// around a `spawn_blocking` call alone would (it would not - see
/// `ScanControl`'s own doc comment). Each row actually scored also
/// advances `control`'s progress count (review 161 #3).
pub fn find_similar_pairs(
    // path, thumbnail_path, features
    map: &[(String, Option<String>, Vec<f32>)],
    threshold: f32,
    control: &ScanControl,
) -> Vec<SimilarImagePair> {
    let n = map.len();
    if n == 0 {
        return vec![];
    }

    let mut ret: Vec<SimilarImagePair> = (0..n)
        .into_par_iter()
        .flat_map(|i| {
            if control.is_cancelled() {
                return Vec::new();
            }
            let (path_a, thumbnail_path_a, vec_a) = &map[i];
            let mut pairs = Vec::new();
            for (path_b, thumbnail_path_b, vec_b) in map.iter().skip(i + 1) {
                let Some(score) = cosine_dot(vec_a, vec_b) else {
                    continue;
                };
                if score >= threshold {
                    // Clone paths only when a pair is kept.
                    pairs.push((
                        (path_a.clone(), thumbnail_path_a.clone()),
                        (path_b.clone(), thumbnail_path_b.clone()),
                        score,
                    ));
                }
            }
            control.advance();
            pairs
        })
        .collect();

    sort_pairs(&mut ret);

    ret
}

fn cosine_dot(a: &[f32], b: &[f32]) -> Option<f32> {
    if a.is_empty() || a.len() != b.len() {
        return None;
    }

    let score: f32 = a.iter().zip(b).map(|(a, b)| a * b).sum();
    score.is_finite().then_some(score)
}

/// `pub` so a caller merging image and video pairs into one list (RFC
/// 047 S3.3's single shared cap) can re-sort the combined list with the
/// identical ordering, rather than reimplementing or approximating it.
pub fn sort_pairs(pairs: &mut [SimilarImagePair]) {
    pairs.par_sort_unstable_by(compare_pairs);
}

fn compare_pairs(a: &SimilarImagePair, b: &SimilarImagePair) -> Ordering {
    b.2.total_cmp(&a.2)
        .then_with(|| a.0.0.cmp(&b.0.0))
        .then_with(|| a.1.0.cmp(&b.1.0))
}

#[cfg(test)]
mod tests {
    use super::find_similar_pairs;
    use crate::pipeline::score::similarity::scan_control::ScanControl;

    fn item(path: &str, vector: &[f32]) -> (String, Option<String>, Vec<f32>) {
        (path.to_owned(), None, vector.to_vec())
    }

    #[test]
    fn empty_input_returns_no_pairs() {
        let pairs = find_similar_pairs(&[], 0.5, &ScanControl::new());

        assert!(pairs.is_empty());
    }

    #[test]
    fn threshold_is_inclusive_and_pairs_are_unique() {
        let map = vec![
            item("a.jpg", &[1.0, 0.0]),
            item("b.jpg", &[0.8, 0.6]),
            item("c.jpg", &[0.0, 1.0]),
        ];

        let pairs = find_similar_pairs(&map, 0.8, &ScanControl::new());

        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0].0.0, "a.jpg");
        assert_eq!(pairs[0].1.0, "b.jpg");
        assert!((pairs[0].2 - 0.8).abs() < f32::EPSILON);
    }

    /// RFC 047 S3.3: this used to also prove truncation to `max_pairs`
    /// (the third argument this function no longer takes) - capping is
    /// now the caller's job, applied once to a merged image+video list,
    /// so this only has global sort order and stable tie-breaking left
    /// to prove.
    #[test]
    fn output_is_globally_sorted_with_stable_tie_ordering() {
        let map = vec![
            item("b.jpg", &[1.0, 0.0]),
            item("a.jpg", &[1.0, 0.0]),
            item("c.jpg", &[1.0, 0.0]),
            item("d.jpg", &[0.0, 1.0]),
        ];

        let pairs = find_similar_pairs(&map, 0.0, &ScanControl::new());

        let keys = pairs
            .iter()
            .map(|pair| (pair.0.0.as_str(), pair.1.0.as_str(), pair.2))
            .collect::<Vec<_>>();
        assert_eq!(
            keys,
            vec![
                ("a.jpg", "c.jpg", 1.0),
                ("b.jpg", "a.jpg", 1.0),
                ("b.jpg", "c.jpg", 1.0),
                ("a.jpg", "d.jpg", 0.0),
                ("b.jpg", "d.jpg", 0.0),
                ("c.jpg", "d.jpg", 0.0),
            ]
        );
    }

    #[test]
    fn invalid_vectors_are_skipped() {
        let map = vec![
            item("a.jpg", &[1.0, 0.0]),
            item("b.jpg", &[1.0]),
            item("c.jpg", &[f32::NAN, 0.0]),
        ];

        let pairs = find_similar_pairs(&map, 0.0, &ScanControl::new());

        assert!(pairs.is_empty());
    }

    #[test]
    fn a_pre_cancelled_control_returns_no_pairs_and_advances_nothing() {
        let map = vec![
            item("a.jpg", &[1.0, 0.0]),
            item("b.jpg", &[1.0, 0.0]),
            item("c.jpg", &[1.0, 0.0]),
        ];
        let control = ScanControl::new();
        control.cancel();

        let pairs = find_similar_pairs(&map, 0.0, &control);

        assert!(pairs.is_empty());
        assert_eq!(
            control.snapshot().0,
            0,
            "a row skipped for cancellation must not count as compared"
        );
    }

    /// Review 161 #5's own fix (a deterministic trigger, not a sleep)
    /// applied to the new cancellation path itself: cancels once real
    /// progress is observed, by polling `ScanControl::snapshot` from
    /// another thread rather than waiting a fixed duration - the busy
    /// loop reacts to actual work done, not wall-clock time, so it
    /// cannot be flaky on a faster or slower machine the way a sleep-
    /// then-cancel race would be.
    #[test]
    fn cancelling_once_progress_is_observed_stops_short_of_the_full_scan() {
        // Enough rows that the scan cannot finish before the polling
        // thread below observes at least one row and cancels.
        let n = 400;
        let map: Vec<(String, Option<String>, Vec<f32>)> = (0..n)
            .map(|i| item(&format!("image-{i}.jpg"), &[1.0, 0.0]))
            .collect();
        let control = std::sync::Arc::new(ScanControl::new());

        let canceller = {
            let control = control.clone();
            std::thread::spawn(move || {
                while control.snapshot().0 == 0 {
                    std::hint::spin_loop();
                }
                control.cancel();
            })
        };

        let pairs = find_similar_pairs(&map, 0.0, &control);
        canceller.join().unwrap();

        let (compared, _) = control.snapshot();
        assert!(
            compared < n,
            "expected the scan to stop before every row was compared, got all {n}"
        );
        assert!(
            compared > 0,
            "expected at least the triggering row to count"
        );
        // Every row that *was* compared still produced correct pairs -
        // cancellation must not corrupt the rows it did not skip.
        for (left, right, score) in &pairs {
            assert!(left.0 != right.0, "{left:?}/{right:?}");
            assert!((*score - 1.0).abs() < f32::EPSILON);
        }
    }

    #[test]
    #[ignore = "release-mode performance smoke for RFC 022 review evidence"]
    fn performance_smoke_exact_search_600_by_512() {
        let map = (0..600)
            .map(|i| {
                let vector = (0..512)
                    .map(|j| {
                        let value = ((i * 31 + j * 17) % 97) as f32 / 97.0;
                        value - 0.5
                    })
                    .collect::<Vec<_>>();
                item(&format!("image-{i:04}.jpg"), &vector)
            })
            .collect::<Vec<_>>();

        let started = std::time::Instant::now();
        let pairs = find_similar_pairs(&map, 0.0, &ScanControl::new());
        let elapsed = started.elapsed();

        println!(
            "exact image similarity smoke: {} vectors x 512 dims, {} pairs found (uncapped), \
             {:?}",
            map.len(),
            pairs.len(),
            elapsed
        );
    }
}
