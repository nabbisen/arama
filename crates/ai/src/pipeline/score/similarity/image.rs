use std::{cmp::Ordering, path::PathBuf};

use hnsw_rs::{hnsw::Hnsw, prelude::DistL2};
use rayon::{
    iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator},
    slice::ParallelSliceMut,
};

// 前提: キャッシュ保存時に L2 正規化済
pub async fn find_similar_pairs(
    map: &Vec<(PathBuf, Vec<f32>)>,
    threshold: f32,
    k_neighbors: usize, // 各画像について何件の近傍を探すか. 少し余裕を持った値（例：50〜100）に設定しておくのが安全
) -> Vec<(PathBuf, PathBuf, f32)> {
    let n = map.len();
    if n == 0 {
        return vec![];
    }

    // 1. HNSWインデックスの構築
    let hnsw = Hnsw::<f32, DistL2>::new(16, n, 16, 200, DistL2);

    // map 内の Vec<f32> への「参照」をそのまま HNSW に登録する
    // これにより、数千〜数万件の大きなベクトルのコピーが完全にゼロになります
    let data_with_id: Vec<(&Vec<f32>, usize)> =
        map.iter().enumerate().map(|(i, (_, v))| (v, i)).collect();

    hnsw.parallel_insert(&data_with_id);

    // 2. 検索とフィルタリング
    let ef_search = 100;
    let mut results: Vec<(PathBuf, PathBuf, f32)> = (0..n)
        .into_par_iter()
        .flat_map(|i| {
            let (path_a, vec_a) = &map[i]; // 元のデータを参照
            let neighbors = hnsw.search(vec_a, k_neighbors, ef_search);

            let mut pairs = Vec::new();
            for neighbor in neighbors {
                let j = neighbor.d_id;
                if i < j {
                    let (path_b, vec_b) = &map[j]; // 相手も参照

                    // ドット積計算
                    let score: f32 = vec_a.iter().zip(vec_b).map(|(a, b)| a * b).sum();

                    if score >= threshold {
                        // ここで初めて PathBuf をクローンして結果リストに入れる
                        pairs.push((path_a.clone(), path_b.clone(), score));
                    }
                }
            }
            pairs
        })
        .collect();

    // 3. ソート
    results.par_sort_unstable_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(Ordering::Equal));

    results
}
