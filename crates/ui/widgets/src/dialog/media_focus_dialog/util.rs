use std::path::Path;

use rayon::prelude::*;

use arama_cache::{CacheConfig, DbLocation, ImageCacheConfig, ImageCacheReader};
use arama_env::{
    MIN_IMAGE_SIMILARITY, VIDEO_EXTENSION_ALLOWLIST, cache_storage_path, cache_thumbnail_dir_path,
};

use super::types::SimilarMediaItem;

pub fn similar_media(path: &Path) -> Vec<SimilarMediaItem> {
    let is_video = path.extension().is_some_and(|x| {
        VIDEO_EXTENSION_ALLOWLIST.contains(&x.to_string_lossy().to_string().as_str())
    });

    // todo
    if is_video {
        return vec![];
    }

    similar_images(path)
}

fn similar_images(path: &Path) -> Vec<SimilarMediaItem> {
    let db_location =
        DbLocation::Custom(cache_storage_path().expect("failed to get cache stogate path"));
    let read_conns = 4;
    let thumbnail_dir =
        Some(cache_thumbnail_dir_path().expect("failed to get cache thumbnail dir path"));
    let cache_config = CacheConfig {
        db_location,
        read_conns,
        thumbnail_dir,
    };

    let image_cache_reader = ImageCacheReader::as_session(ImageCacheConfig {
        cache_config: cache_config.clone(),
    })
    .expect("failed to get image cache writer");

    let cache_entries = image_cache_reader
        .all_in_dir(&path)
        .expect("failed to lookup")
        .into_iter()
        // todo
        .filter(|x| x.is_ok())
        .map(|x| x.unwrap())
        .collect::<Vec<_>>();

    // 比較元と比較対象を分離
    let (target_item, candidates): (Vec<_>, Vec<_>) = cache_entries.into_iter().partition(|x| {
        &x.path
            == &path
                .canonicalize()
                .expect("failed to canonicalize")
                .to_string_lossy()
                .to_string()
    });

    let target_vec = &target_item[0]
        .features
        .as_ref()
        .expect("failed to get target features")
        .clip_vector;

    // 類似度計算を並列で実行
    let mut ret = candidates
        .into_par_iter() // Rayonで並列化
        .map(|x| {
            let similarity = dot_product(
                &target_vec,
                &x.features.expect("failed to get features").clip_vector,
            );
            SimilarMediaItem {
                path: x.path,
                thumbnail_path: x.thumbnail_path,
                similarity,
            }
        })
        .filter(|x| MIN_IMAGE_SIMILARITY <= x.similarity)
        .collect::<Vec<_>>();

    // 類似度の降順でソート（不安定ソートの方が高速）
    ret.sort_unstable_by(|a, b| {
        b.similarity
            .partial_cmp(&a.similarity)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    ret
}

/// 2つのベクトルの内積を計算（正規化済みならこれがコサイン類似度）
fn dot_product(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

// let mut ret: Vec<(PathBuf, f32)> = vec![];

// let mut image_path_embeddings: Vec<(PathBuf, Vec<f32>)> = vec![];
// let is_video = path.extension().is_some_and(|x| {
//     VIDEO_EXTENSION_ALLOWLIST.contains(&x.to_string_lossy().to_string().as_str())
// });
// if is_video {
//     let video_cache_reader = VideoCacheReader::as_session(VideoCacheConfig {
//         cache_config,
//         ffmpeg_path: Some(
//             VideoEngine::ffmpeg_path().expect("failed to get ffmpeg path"),
//         ),
//     })
//     .expect("failed to get video cache writer");

//     let feature = match video_cache_reader.lookup(&path).expect("failed to lookup")
//     {
//         LookupResult::Hit(x) => Some((
//             PathBuf::from(
//                 // todo
//                 x.thumbnail_path.unwrap_or_default(),
//             ),
//             x.features
//                 .expect("failed to get feature")
//                 .clip_vector
//                 .expect("failed to get video clip embedding list"),
//         )),
//         _ => {
//             // todo: error handling
//             None
//         }
//     };

//     if let Some(feature) = feature {
//         video_path_embeddings.push(feature);
//     }
// } else {
//     let image_cache_reader = ImageCacheReader::as_session(ImageCacheConfig {
//         cache_config: cache_config.clone(),
//     })
//     .expect("failed to get image cache writer");

//     let feature = match image_cache_reader.lookup(&path).expect("failed to lookup")
//     {
//         LookupResult::Hit(x) => Some((
//             PathBuf::from(x.thumbnail_path.expect("failed to get thumbnail path")),
//             x.features.expect("failed to get feature").clip_vector,
//         )),
//         _ => {
//             // todo: error handling
//             None
//         }
//     };

//     if let Some(feature) = feature {
//         image_path_embeddings.push(feature);
//     }
// }

// // todo ui sliders for these param(s): threshold (also k_neighbors ?)
// let mut image_pairs = find_similar_pairs(&image_path_embeddings, 0.86, 50).await;
// let video_pairs = find_similar_pairs(&video_path_embeddings, 0.86, 50).await;

// ret
