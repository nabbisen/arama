use std::{cmp::Ordering, path::PathBuf};

use hnsw_rs::{hnsw::Hnsw, prelude::DistL2};
use rayon::{
    iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator},
    slice::ParallelSliceMut,
};

// 前提: キャッシュ保存時に L2 正規化済
pub async fn image_image_find_similar_pairs(
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

pub async fn video_video_find_similar_pairs(
    map: &Vec<(PathBuf, Vec<Vec<f32>>)>, // 入力はフレームリストのベクトル
    threshold: f32,
    k_neighbors: usize,
) -> Vec<(PathBuf, PathBuf, f32)> {
    let n = map.len();
    if n == 0 {
        return vec![];
    }

    // 1. 各動画の「平均特徴量ベクトル」を計算する (Mean Pooling)
    // HNSWに登録するために、各動画を1つの Vec<f32> に変換します。
    let pooled_features: Vec<Vec<f32>> = map
        .par_iter()
        .map(|(_, frames)| {
            if frames.is_empty() {
                return vec![];
            }
            let dim = frames[0].len();
            let mut mean_vec = vec![0.0; dim];
            for frame in frames {
                for (i, val) in frame.iter().enumerate() {
                    mean_vec[i] += val;
                }
            }
            let f_n = frames.len() as f32;
            for val in &mut mean_vec {
                *val /= f_n;
            }
            // ここで L2正規化 をしておくと、後のドット積がそのままコサイン類似度になります
            let norm = mean_vec.iter().map(|x| x * x).sum::<f32>().sqrt();
            if norm > 0.0 {
                for val in &mut mean_vec {
                    *val /= norm;
                }
            }
            mean_vec
        })
        .collect();

    // 2. HNSWインデックスの構築
    // DistL2 を使用。正規化済みベクトルの場合、L2距離の最小化はコサイン類似度の最大化と等価です。
    let hnsw = Hnsw::<f32, DistL2>::new(16, n, 16, 200, DistL2);

    let data_with_id: Vec<(&Vec<f32>, usize)> = pooled_features
        .iter()
        .enumerate()
        .map(|(i, v)| (v, i))
        .collect();

    hnsw.parallel_insert(&data_with_id);

    // 3. 検索とフィルタリング
    let ef_search = 100;
    let mut results: Vec<(PathBuf, PathBuf, f32)> = (0..n)
        .into_par_iter()
        .flat_map(|i| {
            let vec_a = &pooled_features[i];
            let neighbors = hnsw.search(vec_a, k_neighbors, ef_search);

            let mut pairs = Vec::new();
            for neighbor in neighbors {
                let j = neighbor.d_id;
                // 重複ペア(i,j)と(j,i)を防ぎ、自分自身(i==j)を除外する
                if i < j {
                    let vec_b = &pooled_features[j];

                    // すでに正規化済みなので、ドット積 ＝ コサイン類似度
                    let score: f32 = vec_a.iter().zip(vec_b).map(|(a, b)| a * b).sum();

                    if score >= threshold {
                        pairs.push((map[i].0.clone(), map[j].0.clone(), score));
                    }
                }
            }
            pairs
        })
        .collect();

    // 4. ソート（類似度が高い順）
    results.par_sort_unstable_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(Ordering::Equal));

    results
}
