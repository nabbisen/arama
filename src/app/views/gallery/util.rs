use std::path::PathBuf;

use swdir::{DirNode, Recurse};

use crate::engine::{
    pipeline::clip::inference::{clip, clip_calculator},
    store::file::file_embedding_map::FileEmbeddingMap,
};

// フォルダ内の画像を非同期で検索するヘルパー関数
pub async fn load_images(path: PathBuf, swdir_depth_limit: Option<usize>) -> DirNode {
    const EXTENSION_ALLOWLIST: &[&str; 6] = &["png", "jpg", "jpeg", "webp", "gif", "bmp"];

    let dir_node = swdir::Swdir::default()
        .set_root_path(path)
        .set_recurse(Recurse {
            enabled: true,
            depth_limit: swdir_depth_limit,
        })
        .set_extension_allowlist(EXTENSION_ALLOWLIST)
        .expect("failed to set extension allowlist")
        .scan();

    dir_node
}

// フォルダ内の画像を非同期で検索するヘルパー関数
pub async fn calculate_embedding(
    dir_node: DirNode,
    threshold: f32,
) -> (FileEmbeddingMap, Vec<(PathBuf, PathBuf, f32)>) {
    let calculator = clip_calculator().expect("failed to load clip calculator");

    let mut map = FileEmbeddingMap::default();
    dir_node.flatten_paths().iter().for_each(|path| {
        let file_embedding = clip(path, &calculator).expect("failed to clip calculation");
        map.set_embedding(&file_embedding);
    });

    let similar_pairs = map
        .similar_pairs(threshold)
        .expect("failed to get similar pairs");

    (map, similar_pairs)
}
