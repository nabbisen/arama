use std::cmp::Ordering;

use rayon::{
    iter::{IntoParallelIterator, ParallelIterator},
    slice::ParallelSliceMut,
};

// Assumption: vectors were L2-normalized when cached.
// Return: left (path, thumbnail_path), right (path, thumbnail_path), similarity.
pub type MediaRef = (String, Option<String>);
pub type SimilarImagePair = (MediaRef, MediaRef, f32);

pub fn find_similar_pairs(
    // path, thumbnail_path, features
    map: &[(String, Option<String>, Vec<f32>)],
    threshold: f32,
    max_pairs: usize,
) -> Vec<SimilarImagePair> {
    let n = map.len();
    if n == 0 || max_pairs == 0 {
        return vec![];
    }

    let mut ret: Vec<SimilarImagePair> = (0..n)
        .into_par_iter()
        .flat_map(|i| {
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
            sort_pairs(&mut pairs);
            pairs.truncate(max_pairs);
            pairs
        })
        .collect();

    sort_pairs(&mut ret);
    ret.truncate(max_pairs);

    ret
}

fn cosine_dot(a: &[f32], b: &[f32]) -> Option<f32> {
    if a.is_empty() || a.len() != b.len() {
        return None;
    }

    let score: f32 = a.iter().zip(b).map(|(a, b)| a * b).sum();
    score.is_finite().then_some(score)
}

fn sort_pairs(pairs: &mut [SimilarImagePair]) {
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

    fn item(path: &str, vector: &[f32]) -> (String, Option<String>, Vec<f32>) {
        (path.to_owned(), None, vector.to_vec())
    }

    #[test]
    fn empty_input_returns_no_pairs() {
        let pairs = find_similar_pairs(&[], 0.5, 10);

        assert!(pairs.is_empty());
    }

    #[test]
    fn threshold_is_inclusive_and_pairs_are_unique() {
        let map = vec![
            item("a.jpg", &[1.0, 0.0]),
            item("b.jpg", &[0.8, 0.6]),
            item("c.jpg", &[0.0, 1.0]),
        ];

        let pairs = find_similar_pairs(&map, 0.8, 10);

        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0].0.0, "a.jpg");
        assert_eq!(pairs[0].1.0, "b.jpg");
        assert!((pairs[0].2 - 0.8).abs() < f32::EPSILON);
    }

    #[test]
    fn output_is_global_top_n_with_stable_tie_ordering() {
        let map = vec![
            item("b.jpg", &[1.0, 0.0]),
            item("a.jpg", &[1.0, 0.0]),
            item("c.jpg", &[1.0, 0.0]),
            item("d.jpg", &[0.0, 1.0]),
        ];

        let pairs = find_similar_pairs(&map, 0.0, 3);

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
            ]
        );
    }

    #[test]
    fn zero_limit_returns_no_pairs() {
        let map = vec![item("a.jpg", &[1.0]), item("b.jpg", &[1.0])];

        let pairs = find_similar_pairs(&map, 0.0, 0);

        assert!(pairs.is_empty());
    }

    #[test]
    fn invalid_vectors_are_skipped() {
        let map = vec![
            item("a.jpg", &[1.0, 0.0]),
            item("b.jpg", &[1.0]),
            item("c.jpg", &[f32::NAN, 0.0]),
        ];

        let pairs = find_similar_pairs(&map, 0.0, 10);

        assert!(pairs.is_empty());
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
        let pairs = find_similar_pairs(&map, 0.0, 50);
        let elapsed = started.elapsed();

        println!(
            "exact image similarity smoke: {} vectors x 512 dims, {} pairs retained, {:?}",
            map.len(),
            pairs.len(),
            elapsed
        );
        assert!(pairs.len() <= 50);
    }
}
