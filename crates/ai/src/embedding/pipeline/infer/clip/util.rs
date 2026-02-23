use std::{cmp::Ordering, path::PathBuf};

use hnsw_rs::{hnsw::Hnsw, prelude::DistL2};
use rayon::{
    iter::{IntoParallelIterator, IntoParallelRefMutIterator, ParallelIterator},
    slice::ParallelSliceMut,
};

pub async fn find_similar_pairs_efficient(
    map: &Vec<(PathBuf, Vec<f32>)>,
    threshold: f32,
    k_neighbors: usize, // 各画像について何件の近傍を探すか
) -> Vec<(PathBuf, PathBuf, f32)> {
    let paths: Vec<PathBuf> = map.iter().map(|x| x.0.to_owned()).collect();
    let mut vectors: Vec<Vec<f32>> = map.iter().map(|x| x.1.to_owned()).collect();
    let n = vectors.len();
    if n == 0 {
        return vec![];
    }

    // 1. L2正規化 (Rayonで並列処理)
    // これにより「ドット積 = コサイン類似度」になる
    vectors.par_iter_mut().for_each(|v| {
        let norm = v.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 1e-8 {
            v.iter_mut().for_each(|x| *x /= norm);
        }
    });

    // 2. HNSWインデックスの構築
    let hnsw = Hnsw::<f32, DistL2>::new(16, n, 16, 200, DistL2);

    let data_with_id: Vec<(&Vec<f32>, usize)> =
        vectors.iter().enumerate().map(|(i, v)| (v, i)).collect();

    hnsw.parallel_insert(&data_with_id);

    // 3. 検索とフィルタリング
    let ef_search = 100; // 探索範囲（大きいほど高精度・低速）
    let mut results: Vec<(PathBuf, PathBuf, f32)> = (0..n)
        .into_par_iter()
        .flat_map(|i| {
            let query = &vectors[i];

            // DistL2で近傍を検索 (近い順にK件)
            let neighbors = hnsw.search(query, k_neighbors, ef_search);

            let mut pairs = Vec::new();
            for neighbor in neighbors {
                let j = neighbor.d_id;

                // 重複 (B-A) と自己マッチ (A-A) を除外
                if i < j {
                    // 正確なコサイン類似度（内積）を自前で再計算
                    // (正規化済みなので要素同士の積の和だけでOK)
                    let score: f32 = query.iter().zip(&vectors[j]).map(|(a, b)| a * b).sum();

                    // ここで指定した threshold (例: 0.85) を使って判定
                    if score >= threshold {
                        pairs.push((paths[i].clone(), paths[j].clone(), score));
                    }
                }
            }
            pairs
        })
        .collect();

    // 4. スコアで降順ソート
    results.par_sort_unstable_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(Ordering::Equal));

    results
}
