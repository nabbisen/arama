use std::path::PathBuf;

// フォルダ内の画像を非同期で検索するヘルパー関数
pub async fn load_images(dir: &str) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Ok(mut entries) = tokio::fs::read_dir(dir).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if path.is_file() {
                // 拡張子で画像のみをフィルタリング
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    match ext.to_lowercase().as_str() {
                        "png" | "jpg" | "jpeg" | "webp" | "gif" | "bmp" => paths.push(path),
                        _ => {}
                    }
                }
            }
        }
    }
    paths.sort(); // 名前順にソート
    paths
}
