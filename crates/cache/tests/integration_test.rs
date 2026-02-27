//! インテグレーションテスト。
//!
//! identity の計算はキャッシュストア内部で行われるため、
//! テスト用の実ファイルを tempfile で作成して使う。

use std::io::Write;
use std::path::PathBuf;

use arama_cache::{CacheStore, LookupResult, UpsertImageRequest, UpsertVideoRequest};

// ---------------------------------------------------------------------------
// テストヘルパー
// ---------------------------------------------------------------------------

struct TempFile {
    path: PathBuf,
}

impl TempFile {
    fn new(content: &[u8]) -> Self {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        f.write_all(content).unwrap();
        // keep パスを保持し tempfile は削除させない
        let path = f.path().to_path_buf();
        f.keep().unwrap();
        TempFile { path }
    }

    fn path_str(&self) -> &str {
        self.path.to_str().unwrap()
    }

    /// ファイルの内容を書き換えてタイムスタンプも変更する
    fn overwrite(&self, content: &[u8]) {
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&self.path)
            .unwrap();
        f.write_all(content).unwrap();
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

// ---------------------------------------------------------------------------
// 画像系テスト
// ---------------------------------------------------------------------------

#[test]
fn image_miss_on_empty_db() {
    let store = CacheStore::open_in_memory().unwrap();
    let f = TempFile::new(b"dummy image data");

    let result = store.lookup_image(f.path_str()).unwrap();
    assert!(matches!(result, LookupResult::Miss));
}

#[test]
fn image_hit_after_upsert() {
    let store = CacheStore::open_in_memory().unwrap();
    let f = TempFile::new(b"image content");

    store
        .upsert_image(UpsertImageRequest {
            file_path: f.path_str().to_string(),
            thumbnail_path: Some("/cache/thumb.jpg".to_string()),
            clip_vector: Some(vec![1.0, 2.0, 3.0]),
        })
        .unwrap();

    match store.lookup_image(f.path_str()).unwrap() {
        LookupResult::Hit(entry) => {
            assert_eq!(entry.thumbnail_path.unwrap(), "/cache/thumb.jpg");
            assert_eq!(entry.features.unwrap().clip_vector, vec![1.0f32, 2.0, 3.0]);
        }
        other => panic!("expected Hit, got {:?}", other),
    }
}

#[test]
fn image_invalidated_when_content_changes() {
    let store = CacheStore::open_in_memory().unwrap();
    let f = TempFile::new(b"original content");

    store
        .upsert_image(UpsertImageRequest {
            file_path: f.path_str().to_string(),
            thumbnail_path: Some("/thumb.jpg".to_string()),
            clip_vector: Some(vec![0.5]),
        })
        .unwrap();

    // ファイルを書き換え
    f.overwrite(b"completely different content!!");

    let result = store.lookup_image(f.path_str()).unwrap();
    assert!(matches!(result, LookupResult::Invalidated));

    // 古いデータが消えていること
    let result2 = store.lookup_image(f.path_str()).unwrap();
    assert!(matches!(result2, LookupResult::Miss));
}

#[test]
fn image_partial_upsert_preserves_existing() {
    let store = CacheStore::open_in_memory().unwrap();
    let f = TempFile::new(b"some image");

    // 初回: thumbnail だけ
    store
        .upsert_image(UpsertImageRequest {
            file_path: f.path_str().to_string(),
            thumbnail_path: Some("/thumb.jpg".to_string()),
            clip_vector: None,
        })
        .unwrap();

    // 2 回目: clip_vector だけ追加
    store
        .upsert_image(UpsertImageRequest {
            file_path: f.path_str().to_string(),
            thumbnail_path: None,
            clip_vector: Some(vec![9.0, 8.0]),
        })
        .unwrap();

    match store.lookup_image(f.path_str()).unwrap() {
        LookupResult::Hit(entry) => {
            assert_eq!(entry.thumbnail_path.unwrap(), "/thumb.jpg");
            assert_eq!(entry.features.unwrap().clip_vector, vec![9.0f32, 8.0]);
        }
        other => panic!("{:?}", other),
    }
}

// ---------------------------------------------------------------------------
// 動画系テスト
// ---------------------------------------------------------------------------

#[test]
fn video_hit_after_upsert() {
    let store = CacheStore::open_in_memory().unwrap();
    let f = TempFile::new(b"fake video data");

    store
        .upsert_video(UpsertVideoRequest {
            file_path: f.path_str().to_string(),
            thumbnail_path: Some("/thumb/clip.jpg".to_string()),
            clip_vector: Some(vec![1.0, 2.0]),
            wav2vec2_vector: Some(vec![3.0, 4.0]),
        })
        .unwrap();

    match store.lookup_video(f.path_str()).unwrap() {
        LookupResult::Hit(entry) => {
            let feat = entry.features.unwrap();
            assert_eq!(feat.clip_vector, vec![1.0f32, 2.0]);
            assert_eq!(feat.wav2vec2_vector, vec![3.0f32, 4.0]);
        }
        other => panic!("{:?}", other),
    }
}

#[test]
fn video_invalidated_when_content_changes() {
    let store = CacheStore::open_in_memory().unwrap();
    let f = TempFile::new(b"original video");

    store
        .upsert_video(UpsertVideoRequest {
            file_path: f.path_str().to_string(),
            thumbnail_path: None,
            clip_vector: Some(vec![0.1]),
            wav2vec2_vector: Some(vec![0.2]),
        })
        .unwrap();

    f.overwrite(b"modified video content that is different");

    assert!(matches!(
        store.lookup_video(f.path_str()).unwrap(),
        LookupResult::Invalidated
    ));
}

// ---------------------------------------------------------------------------
// 汎用 API テスト
// ---------------------------------------------------------------------------

#[test]
fn delete_removes_entry() {
    let store = CacheStore::open_in_memory().unwrap();
    let f = TempFile::new(b"data");

    store
        .upsert_image(UpsertImageRequest {
            file_path: f.path_str().to_string(),
            thumbnail_path: Some("/t.jpg".to_string()),
            clip_vector: Some(vec![1.0]),
        })
        .unwrap();

    assert!(store.delete(f.path_str()).unwrap());
    assert!(!store.delete(f.path_str()).unwrap()); // 2 回目は false

    assert!(matches!(
        store.lookup_image(f.path_str()).unwrap(),
        LookupResult::Miss
    ));
}

#[test]
fn list_paths_returns_all() {
    let store = CacheStore::open_in_memory().unwrap();
    let files: Vec<_> = (0..3)
        .map(|i| TempFile::new(format!("content{i}").as_bytes()))
        .collect();

    for f in &files {
        store
            .upsert_image(UpsertImageRequest {
                file_path: f.path_str().to_string(),
                thumbnail_path: None,
                clip_vector: None,
            })
            .unwrap();
    }

    let paths = store.list_paths().unwrap();
    assert_eq!(paths.len(), 3);
}

#[test]
fn verify_or_invalidate_returns_false_and_clears() {
    let store = CacheStore::open_in_memory().unwrap();
    let f = TempFile::new(b"original");

    store
        .upsert_image(UpsertImageRequest {
            file_path: f.path_str().to_string(),
            thumbnail_path: None,
            clip_vector: Some(vec![0.0]),
        })
        .unwrap();

    f.overwrite(b"changed!!");

    assert!(!store.verify_or_invalidate(f.path_str()).unwrap());
    assert!(store.list_paths().unwrap().is_empty());
}

// ---------------------------------------------------------------------------
// HashStrategy テスト
// ---------------------------------------------------------------------------

#[test]
fn hash_strategy_full_works() {
    use arama_cache::identity::hash_strategy::HashStrategy;
    use arama_cache::store::cashe_store_config::CacheStoreConfig;

    let store = CacheStore::open_in_memory_with_config(CacheStoreConfig {
        read_conns: None,
        hash_strategy: HashStrategy::Full,
    })
    .unwrap();

    let f = TempFile::new(b"small file with full hash strategy");

    store
        .upsert_image(UpsertImageRequest {
            file_path: f.path_str().to_string(),
            thumbnail_path: None,
            clip_vector: Some(vec![1.0]),
        })
        .unwrap();

    assert!(matches!(
        store.lookup_image(f.path_str()).unwrap(),
        LookupResult::Hit(_)
    ));

    f.overwrite(b"different content");
    assert!(matches!(
        store.lookup_image(f.path_str()).unwrap(),
        LookupResult::Invalidated
    ));
}
