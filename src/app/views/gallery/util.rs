use std::path::PathBuf;

use swdir::{DirNode, Recurse};

// フォルダ内の画像を非同期で検索するヘルパー関数
pub async fn load_images(path: PathBuf) -> DirNode {
    const EXTENSION_ALLOWLIST: &[&str; 6] = &["png", "jpg", "jpeg", "webp", "gif", "bmp"];

    let dir_node = swdir::Swdir::default()
        .set_root_path(path)
        .set_recurse(Recurse {
            enabled: true,
            skip_hidden: true,
            depth_limit: None,
        })
        .set_extension_allowlist(EXTENSION_ALLOWLIST)
        .expect("failed to set extension allowlist")
        .scan();

    dir_node
}
