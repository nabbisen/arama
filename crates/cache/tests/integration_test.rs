//! インテグレーションテスト。
//!
//! 実ファイルを tempfile で作成して使う。

use std::io::Write;
use std::path::PathBuf;

use arama_cache::{
    CacheConfig, CacheWriter, HashStrategy, LookupResult, UpsertImageRequest, UpsertVideoRequest,
    config::db_location::DbLocation,
};

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
        let path = f.path().to_path_buf();
        f.keep().unwrap();
        TempFile { path }
    }

    fn path_str(&self) -> &str {
        self.path.to_str().unwrap()
    }

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
    let writer = CacheWriter::open_in_memory().unwrap();
    let f = TempFile::new(b"dummy image data");
    assert!(matches!(
        writer.lookup_image(f.path_str()).unwrap(),
        LookupResult::Miss
    ));
}

#[test]
fn image_hit_after_upsert() {
    let writer = CacheWriter::open_in_memory().unwrap();
    let f = TempFile::new(b"image content");

    writer
        .upsert_image(UpsertImageRequest {
            file_path: f.path_str().to_string(),
            thumbnail_path: Some("/cache/thumb.jpg".to_string()),
            clip_vector: Some(vec![1.0, 2.0, 3.0]),
        })
        .unwrap();

    match writer.lookup_image(f.path_str()).unwrap() {
        LookupResult::Hit(entry) => {
            assert_eq!(entry.thumbnail_path.unwrap(), "/cache/thumb.jpg");
            assert_eq!(entry.features.unwrap().clip_vector, vec![1.0f32, 2.0, 3.0]);
        }
        other => panic!("expected Hit, got {:?}", other),
    }
}

#[test]
fn image_invalidated_when_content_changes() {
    let writer = CacheWriter::open_in_memory().unwrap();
    let f = TempFile::new(b"original content");

    writer
        .upsert_image(UpsertImageRequest {
            file_path: f.path_str().to_string(),
            thumbnail_path: Some("/thumb.jpg".to_string()),
            clip_vector: Some(vec![0.5]),
        })
        .unwrap();

    f.overwrite(b"completely different content!!");

    assert!(matches!(
        writer.lookup_image(f.path_str()).unwrap(),
        LookupResult::Invalidated
    ));
    // 削除後は Miss
    assert!(matches!(
        writer.lookup_image(f.path_str()).unwrap(),
        LookupResult::Miss
    ));
}

#[test]
fn image_partial_upsert_preserves_existing() {
    let writer = CacheWriter::open_in_memory().unwrap();
    let f = TempFile::new(b"some image");

    writer
        .upsert_image(UpsertImageRequest {
            file_path: f.path_str().to_string(),
            thumbnail_path: Some("/thumb.jpg".to_string()),
            clip_vector: None,
        })
        .unwrap();

    writer
        .upsert_image(UpsertImageRequest {
            file_path: f.path_str().to_string(),
            thumbnail_path: None,
            clip_vector: Some(vec![9.0, 8.0]),
        })
        .unwrap();

    match writer.lookup_image(f.path_str()).unwrap() {
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
    let writer = CacheWriter::open_in_memory().unwrap();
    let f = TempFile::new(b"fake video data");

    writer
        .upsert_video(UpsertVideoRequest {
            file_path: f.path_str().to_string(),
            thumbnail_path: Some("/thumb/clip.jpg".to_string()),
            clip_vector: Some(vec![1.0, 2.0]),
            wav2vec2_vector: Some(vec![3.0, 4.0]),
        })
        .unwrap();

    match writer.lookup_video(f.path_str()).unwrap() {
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
    let writer = CacheWriter::open_in_memory().unwrap();
    let f = TempFile::new(b"original video");

    writer
        .upsert_video(UpsertVideoRequest {
            file_path: f.path_str().to_string(),
            thumbnail_path: None,
            clip_vector: Some(vec![0.1]),
            wav2vec2_vector: Some(vec![0.2]),
        })
        .unwrap();

    f.overwrite(b"modified video content that is different");

    assert!(matches!(
        writer.lookup_video(f.path_str()).unwrap(),
        LookupResult::Invalidated
    ));
}

// ---------------------------------------------------------------------------
// 権限モデルのテスト
// ---------------------------------------------------------------------------

#[test]
fn reader_can_lookup_but_not_write() {
    let writer = CacheWriter::open_in_memory().unwrap();
    let f = TempFile::new(b"shared data");

    writer
        .upsert_image(UpsertImageRequest {
            file_path: f.path_str().to_string(),
            thumbnail_path: Some("/t.jpg".to_string()),
            clip_vector: Some(vec![1.0]),
        })
        .unwrap();

    // reader は Arc<CacheStore> を共有 — 追加の DB 接続なし
    let reader = writer.as_reader();
    assert!(matches!(
        reader.lookup_image(f.path_str()).unwrap(),
        LookupResult::Hit(_)
    ));
    // reader に upsert / delete は生えていない (コンパイル時に保証)
}

#[test]
fn reader_invalidates_on_change() {
    let writer = CacheWriter::open_in_memory().unwrap();
    let f = TempFile::new(b"data");

    writer
        .upsert_image(UpsertImageRequest {
            file_path: f.path_str().to_string(),
            thumbnail_path: None,
            clip_vector: Some(vec![0.0]),
        })
        .unwrap();

    let reader = writer.as_reader();
    f.overwrite(b"changed data");

    // CacheReader でも内部 DELETE が実行される
    assert!(matches!(
        reader.lookup_image(f.path_str()).unwrap(),
        LookupResult::Invalidated
    ));
    assert!(matches!(
        reader.lookup_image(f.path_str()).unwrap(),
        LookupResult::Miss
    ));
}

// ---------------------------------------------------------------------------
// 汎用 API テスト
// ---------------------------------------------------------------------------

#[test]
fn delete_removes_entry() {
    let writer = CacheWriter::open_in_memory().unwrap();
    let f = TempFile::new(b"data");

    writer
        .upsert_image(UpsertImageRequest {
            file_path: f.path_str().to_string(),
            thumbnail_path: Some("/t.jpg".to_string()),
            clip_vector: Some(vec![1.0]),
        })
        .unwrap();

    assert!(writer.delete(f.path_str()).unwrap());
    assert!(!writer.delete(f.path_str()).unwrap());
    assert!(matches!(
        writer.lookup_image(f.path_str()).unwrap(),
        LookupResult::Miss
    ));
}

#[test]
fn list_paths_returns_all() {
    let writer = CacheWriter::open_in_memory().unwrap();
    let files: Vec<_> = (0..3)
        .map(|i| TempFile::new(format!("c{i}").as_bytes()))
        .collect();

    for f in &files {
        writer
            .upsert_image(UpsertImageRequest {
                file_path: f.path_str().to_string(),
                thumbnail_path: None,
                clip_vector: None,
            })
            .unwrap();
    }

    assert_eq!(writer.list_paths().unwrap().len(), 3);
    assert_eq!(writer.as_reader().list_paths().unwrap().len(), 3);
}

#[test]
fn verify_or_invalidate_clears_changed_file() {
    let writer = CacheWriter::open_in_memory().unwrap();
    let f = TempFile::new(b"original");

    writer
        .upsert_image(UpsertImageRequest {
            file_path: f.path_str().to_string(),
            thumbnail_path: None,
            clip_vector: Some(vec![0.0]),
        })
        .unwrap();

    f.overwrite(b"changed!!");

    assert!(!writer.verify_or_invalidate(f.path_str()).unwrap());
    assert!(writer.list_paths().unwrap().is_empty());
}

// ---------------------------------------------------------------------------
// HashStrategy テスト
// ---------------------------------------------------------------------------

#[test]
fn hash_strategy_full_detects_change() {
    let writer = CacheWriter::open_in_memory_with_config(CacheConfig {
        db_location: DbLocation::WorkDir(None),
        read_conns: None,
        hash_strategy: HashStrategy::Full,
    })
    .unwrap();

    let f = TempFile::new(b"small file with full hash strategy");

    writer
        .upsert_image(UpsertImageRequest {
            file_path: f.path_str().to_string(),
            thumbnail_path: None,
            clip_vector: Some(vec![1.0]),
        })
        .unwrap();

    assert!(matches!(
        writer.lookup_image(f.path_str()).unwrap(),
        LookupResult::Hit(_)
    ));

    f.overwrite(b"different content");
    assert!(matches!(
        writer.lookup_image(f.path_str()).unwrap(),
        LookupResult::Invalidated
    ));
}

// ---------------------------------------------------------------------------
// oneshot API のテスト
// (実際の DB ファイルを使うため tempfile で arama_cache_DB を差し替える)
// ---------------------------------------------------------------------------

#[test]
fn convenience_upsert_and_lookup() {
    let db = tempfile::NamedTempFile::new().unwrap();
    let db_path = db.path().to_str().unwrap().to_string();
    // keep して削除されないようにする
    let (_, db_path_buf) = db.keep().unwrap();

    // arama_cache_DB 環境変数でパスを差し替え
    // 注意: テスト並列実行時に他テストの env に干渉しないよう
    //       serial に近い運用が望ましいが、ここでは tempfile 名が一意なので衝突しない
    // SAFETY: シングルスレッド起動直後のテスト内での設定
    unsafe {
        std::env::set_var("arama_cache_DB", &db_path);
    }

    let f = TempFile::new(b"oneshot test image");

    arama_cache::writer::api::oneshot::upsert_image(UpsertImageRequest {
        file_path: f.path_str().to_string(),
        thumbnail_path: Some("/t.jpg".to_string()),
        clip_vector: Some(vec![7.0, 8.0]),
    })
    .unwrap();

    match arama_cache::reader::api::oneshot::lookup_image(f.path_str()).unwrap() {
        LookupResult::Hit(entry) => {
            assert_eq!(entry.thumbnail_path.unwrap(), "/t.jpg");
            assert_eq!(entry.features.unwrap().clip_vector, vec![7.0f32, 8.0]);
        }
        other => panic!("expected Hit, got {:?}", other),
    }

    // 後片付け
    unsafe {
        std::env::remove_var("arama_cache_DB");
    }
    let _ = std::fs::remove_file(&db_path_buf);
}

#[test]
fn convenience_delete() {
    let db = tempfile::NamedTempFile::new().unwrap();
    let db_path = db.path().to_str().unwrap().to_string();
    let (_, db_path_buf) = db.keep().unwrap();
    // SAFETY: シングルスレッド起動直後のテスト内での設定
    unsafe {
        std::env::set_var("arama_cache_DB", &db_path);
    }

    let f = TempFile::new(b"delete test");

    arama_cache::writer::api::oneshot::upsert_image(UpsertImageRequest {
        file_path: f.path_str().to_string(),
        thumbnail_path: None,
        clip_vector: Some(vec![1.0]),
    })
    .unwrap();

    assert!(arama_cache::writer::api::oneshot::delete(f.path_str()).unwrap());
    assert!(matches!(
        arama_cache::reader::api::oneshot::lookup_image(f.path_str()).unwrap(),
        LookupResult::Miss
    ));

    unsafe {
        std::env::remove_var("arama_cache_DB");
    }
    let _ = std::fs::remove_file(&db_path_buf);
}
