use std::cmp::Ordering;

use hnsw_rs::{hnsw::Hnsw, prelude::DistL2};
use rayon::{
    iter::{IntoParallelIterator, ParallelIterator},
    slice::ParallelSliceMut,
};

// Assumption: vectors were L2-normalized when cached.
// Return: left (path, thumbnail_path), right (path, thumbnail_path), similarity.
pub type MediaRef = (String, Option<String>);
pub type SimilarImagePair = (MediaRef, MediaRef, f32);

pub async fn find_similar_pairs(
    // path, thumbnail_path, features
    map: &[(String, Option<String>, Vec<f32>)],
    threshold: f32,
    k_neighbors: usize, // Number of neighbors to search for each image; keep some margin, e.g. 50-100.
) -> Vec<SimilarImagePair> {
    let n = map.len();
    if n == 0 {
        return vec![];
    }

    // 1. Build the HNSW index.
    let hnsw = Hnsw::<f32, DistL2>::new(16, n, 16, 200, DistL2);

    // Insert references to the map's Vec<f32> values directly into HNSW.
    // This avoids copying large vectors even for thousands of images.
    let data_with_id: Vec<(&Vec<f32>, usize)> = map
        .iter()
        .enumerate()
        .map(|(i, (_, _, v))| (v, i))
        .collect();

    hnsw.parallel_insert(&data_with_id);

    // 2. Search and filter.
    let ef_search = 100;
    let mut ret: Vec<SimilarImagePair> = (0..n)
        .into_par_iter()
        .flat_map(|i| {
            let (path_a, thumbnail_path_a, vec_a) = &map[i];
            let neighbors = hnsw.search(vec_a, k_neighbors, ef_search);

            let mut pairs = Vec::new();
            for neighbor in neighbors {
                let j = neighbor.d_id;
                if i < j {
                    let (path_b, thumbnail_path_b, vec_b) = &map[j];

                    // Dot product.
                    let score: f32 = vec_a.iter().zip(vec_b).map(|(a, b)| a * b).sum();

                    if score >= threshold {
                        // Clone paths only when a pair is kept.
                        pairs.push((
                            (path_a.clone(), thumbnail_path_a.clone()),
                            (path_b.clone(), thumbnail_path_b.clone()),
                            score,
                        ));
                    }
                }
            }
            pairs
        })
        .collect();

    // 3. Sort.
    ret.par_sort_unstable_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(Ordering::Equal));

    ret
}
