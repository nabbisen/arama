//! `ai_cache` インテグレーションテスト。

use std::io::Write;
use std::path::{Path, PathBuf};

use arama_cache::{
    ImageCacheConfig, ImageCacheReader, ImageCacheWriter, LookupResult, UpsertImageRequest,
    UpsertVideoRequest, VideoCacheConfig, VideoCacheWriter,
};
use file_feature_cache::{CacheConfig, CacheRead, CacheWrite, DbLocation};

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
        let (_, path) = f.keep().unwrap();
        TempFile { path }
    }
    fn path(&self) -> &Path {
        &self.path
    }
    fn overwrite(&self, content: &[u8]) {
        std::fs::write(&self.path, content).unwrap();
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

fn tmp_db() -> tempfile::NamedTempFile {
    tempfile::NamedTempFile::new().unwrap()
}

fn image_writer_with_db(db: &tempfile::NamedTempFile) -> ImageCacheWriter {
    ImageCacheWriter::as_session(ImageCacheConfig {
        cache_config: CacheConfig {
            db_location: DbLocation::Custom(db.path().to_path_buf()),
            read_conns: 2,
            thumbnail_dir: None,
        },
    })
    .unwrap()
}

fn upsert_image(writer: &ImageCacheWriter, path: &Path) {
    writer
        .upsert(UpsertImageRequest {
            path: path.to_path_buf(),
            clip_vector: None,
        })
        .unwrap();
}

// ---------------------------------------------------------------------------
// 画像 upsert / lookup
// ---------------------------------------------------------------------------

#[test]
fn image_upsert_and_lookup_hit() {
    let db = tmp_db();
    let writer = image_writer_with_db(&db);
    let file = TempFile::new(b"image data");

    writer
        .upsert(UpsertImageRequest {
            path: file.path().to_path_buf(),
            clip_vector: Some(vec![1.0, 2.0, 3.0]),
        })
        .unwrap();

    match writer.lookup(file.path()).unwrap() {
        LookupResult::Hit(entry) => {
            let feat = entry.features.expect("features should be Some");
            assert_eq!(feat.clip_vector, vec![1.0f32, 2.0, 3.0]);
            assert!(entry.thumbnail_path.is_none());
        }
        other => panic!("expected Hit, got {:?}", other),
    }
}

#[test]
fn image_lookup_miss_on_unregistered_file() {
    let db = tmp_db();
    let writer = image_writer_with_db(&db);
    let file = TempFile::new(b"unregistered");
    assert!(matches!(
        writer.lookup(file.path()).unwrap(),
        LookupResult::Miss
    ));
}

#[test]
fn image_lookup_miss_on_nonexistent_file() {
    let db = tmp_db();
    let writer = image_writer_with_db(&db);
    assert!(matches!(
        writer
            .lookup(Path::new("/absolutely/no/such/file.jpg"))
            .unwrap(),
        LookupResult::Miss
    ));
}

#[test]
fn image_lookup_invalidated_on_file_change() {
    let db = tmp_db();
    let writer = image_writer_with_db(&db);
    let file = TempFile::new(b"original");
    upsert_image(&writer, file.path());
    file.overwrite(b"modified");
    assert!(matches!(
        writer.lookup(file.path()).unwrap(),
        LookupResult::Invalidated
    ));
}

// ---------------------------------------------------------------------------
// 画像サムネイル自動生成
// ---------------------------------------------------------------------------

/// 最小有効 JPEG バイト列 (1×1 px)。
const MINIMAL_JPEG: &[u8] = &[
    0xff, 0xd8, 0xff, 0xe0, 0x00, 0x10, 0x4a, 0x46, 0x49, 0x46, 0x00, 0x01, 0x01, 0x00, 0x00, 0x01,
    0x00, 0x01, 0x00, 0x00, 0xff, 0xdb, 0x00, 0x43, 0x00, 0x08, 0x06, 0x06, 0x07, 0x06, 0x05, 0x08,
    0x07, 0x07, 0x07, 0x09, 0x09, 0x08, 0x0a, 0x0c, 0x14, 0x0d, 0x0c, 0x0b, 0x0b, 0x0c, 0x19, 0x12,
    0x13, 0x0f, 0x14, 0x1d, 0x1a, 0x1f, 0x1e, 0x1d, 0x1a, 0x1c, 0x1c, 0x20, 0x24, 0x2e, 0x27, 0x20,
    0x22, 0x2c, 0x23, 0x1c, 0x1c, 0x28, 0x37, 0x29, 0x2c, 0x30, 0x31, 0x34, 0x34, 0x34, 0x1f, 0x27,
    0x39, 0x3d, 0x38, 0x32, 0x3c, 0x2e, 0x33, 0x34, 0x32, 0xff, 0xc0, 0x00, 0x0b, 0x08, 0x00, 0x01,
    0x00, 0x01, 0x01, 0x01, 0x11, 0x00, 0xff, 0xc4, 0x00, 0x1f, 0x00, 0x00, 0x01, 0x05, 0x01, 0x01,
    0x01, 0x01, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x02, 0x03, 0x04,
    0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0xff, 0xc4, 0x00, 0xb5, 0x10, 0x00, 0x02, 0x01, 0x03,
    0x03, 0x02, 0x04, 0x03, 0x05, 0x05, 0x04, 0x04, 0x00, 0x00, 0x01, 0x7d, 0x01, 0x02, 0x03, 0x00,
    0x04, 0x11, 0x05, 0x12, 0x21, 0x31, 0x41, 0x06, 0x13, 0x51, 0x61, 0x07, 0x22, 0x71, 0x14, 0x32,
    0x81, 0x91, 0xa1, 0x08, 0x23, 0x42, 0xb1, 0xc1, 0x15, 0x52, 0xd1, 0xf0, 0x24, 0x33, 0x62, 0x72,
    0x82, 0x09, 0x0a, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2a, 0x34, 0x35,
    0x36, 0x37, 0x38, 0x39, 0x3a, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x4a, 0x53, 0x54, 0x55,
    0x56, 0x57, 0x58, 0x59, 0x5a, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6a, 0x73, 0x74, 0x75,
    0x76, 0x77, 0x78, 0x79, 0x7a, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88, 0x89, 0x8a, 0x92, 0x93, 0x94,
    0x95, 0x96, 0x97, 0x98, 0x99, 0x9a, 0xa2, 0xa3, 0xa4, 0xa5, 0xa6, 0xa7, 0xa8, 0xa9, 0xaa, 0xb2,
    0xb3, 0xb4, 0xb5, 0xb6, 0xb7, 0xb8, 0xb9, 0xba, 0xc2, 0xc3, 0xc4, 0xc5, 0xc6, 0xc7, 0xc8, 0xc9,
    0xca, 0xd2, 0xd3, 0xd4, 0xd5, 0xd6, 0xd7, 0xd8, 0xd9, 0xda, 0xe1, 0xe2, 0xe3, 0xe4, 0xe5, 0xe6,
    0xe7, 0xe8, 0xe9, 0xea, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xff, 0xda,
    0x00, 0x08, 0x01, 0x01, 0x00, 0x00, 0x3f, 0x00, 0xfb, 0x6d, 0xff, 0xd9,
];

#[test]
fn image_thumbnail_generated_to_thumbnail_dir() {
    let thumb_dir = tempfile::TempDir::new().unwrap();
    let db = tmp_db();
    let file = TempFile::new(MINIMAL_JPEG);

    let writer = ImageCacheWriter::as_session(ImageCacheConfig {
        cache_config: CacheConfig {
            db_location: DbLocation::Custom(db.path().to_path_buf()),
            read_conns: 1,
            thumbnail_dir: Some(thumb_dir.path().to_path_buf()),
        },
    })
    .unwrap();

    writer
        .upsert(UpsertImageRequest {
            path: file.path().to_path_buf(),
            clip_vector: None,
        })
        .unwrap();

    match writer.lookup(file.path()).unwrap() {
        LookupResult::Hit(entry) => {
            let thumb = entry.thumbnail_path.expect("thumbnail_path should be Some");
            assert!(
                Path::new(&thumb).exists(),
                "thumbnail file should exist: {thumb}"
            );
            assert!(thumb.ends_with(".jpg"), "expected .jpg extension: {thumb}");
        }
        other => panic!("expected Hit, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// 動画 upsert / lookup
// ---------------------------------------------------------------------------

fn video_writer() -> (VideoCacheWriter, tempfile::NamedTempFile) {
    let db = tmp_db();
    let w = VideoCacheWriter::as_session(VideoCacheConfig {
        cache_config: CacheConfig {
            db_location: DbLocation::Custom(db.path().to_path_buf()),
            read_conns: 2,
            thumbnail_dir: None,
        },
        ffmpeg_path: None,
    })
    .unwrap();
    (w, db)
}

#[test]
fn video_upsert_and_lookup_hit_with_both_vectors() {
    let (writer, _db) = video_writer();
    let file = TempFile::new(b"video data");

    writer
        .upsert(UpsertVideoRequest {
            path: file.path().to_path_buf(),
            clip_vector: Some(vec![1.0, 2.0]),
            // wav2vec2_vector: Some(vec![3.0, 4.0]),
        })
        .unwrap();

    match writer.lookup(file.path()).unwrap() {
        LookupResult::Hit(entry) => {
            let feat = entry.features.expect("features should be Some");
            assert_eq!(feat.clip_vector, Some(vec![1.0f32, 2.0]));
            assert_eq!(feat.wav2vec2_vector, Some(vec![3.0f32, 4.0]));
        }
        other => panic!("expected Hit, got {:?}", other),
    }
}

#[test]
fn video_upsert_partial_vectors_preserved_via_coalesce() {
    let (writer, _db) = video_writer();
    let file = TempFile::new(b"video partial");

    // CLIP だけ書き込む
    writer
        .upsert(UpsertVideoRequest {
            path: file.path().to_path_buf(),
            clip_vector: Some(vec![0.5]),
            // wav2vec2_vector: None,
        })
        .unwrap();
    // wav2vec2 だけ追加
    writer
        .upsert(UpsertVideoRequest {
            path: file.path().to_path_buf(),
            clip_vector: None,
            // wav2vec2_vector: Some(vec![0.9]),
        })
        .unwrap();

    match writer.lookup(file.path()).unwrap() {
        LookupResult::Hit(entry) => {
            let feat = entry.features.unwrap();
            assert_eq!(feat.clip_vector, Some(vec![0.5f32]));
            assert_eq!(feat.wav2vec2_vector, Some(vec![0.9f32]));
        }
        other => panic!("expected Hit, got {:?}", other),
    }
}

#[test]
fn video_lookup_miss_on_unregistered_file() {
    let (writer, _db) = video_writer();
    let file = TempFile::new(b"unregistered video");
    assert!(matches!(
        writer.lookup(file.path()).unwrap(),
        LookupResult::Miss
    ));
}

#[test]
fn video_lookup_invalidated_on_file_change() {
    let (writer, _db) = video_writer();
    let file = TempFile::new(b"original video");
    writer
        .upsert(UpsertVideoRequest {
            path: file.path().to_path_buf(),
            clip_vector: Some(vec![1.0]),
            // wav2vec2_vector: None,
        })
        .unwrap();
    file.overwrite(b"modified video");
    assert!(matches!(
        writer.lookup(file.path()).unwrap(),
        LookupResult::Invalidated
    ));
}

// ---------------------------------------------------------------------------
// delete / list_paths
// ---------------------------------------------------------------------------

#[test]
fn delete_removes_image_entry() {
    let db = tmp_db();
    let writer = image_writer_with_db(&db);
    let file = TempFile::new(b"to delete");
    upsert_image(&writer, file.path());
    assert!(writer.delete(file.path()).unwrap());
    assert!(matches!(
        writer.lookup(file.path()).unwrap(),
        LookupResult::Miss
    ));
}

#[test]
fn list_paths_returns_all_registered() {
    let db = tmp_db();
    let writer = image_writer_with_db(&db);
    let files: Vec<_> = (0..3)
        .map(|i| TempFile::new(format!("f{i}").as_bytes()))
        .collect();
    for f in &files {
        upsert_image(&writer, f.path());
    }
    assert_eq!(writer.list_paths().unwrap().len(), 3);
}

// ---------------------------------------------------------------------------
// onetime — ファイルベース DB への永続化
// ---------------------------------------------------------------------------

#[test]
fn onetime_persists_across_instances() {
    let db = tmp_db();
    let loc = DbLocation::Custom(db.path().to_path_buf());
    let file = TempFile::new(b"onetime test");

    ImageCacheWriter::onetime(loc.clone())
        .unwrap()
        .upsert(UpsertImageRequest {
            path: file.path().to_path_buf(),
            clip_vector: None,
        })
        .unwrap();

    match ImageCacheWriter::onetime(loc)
        .unwrap()
        .lookup(file.path())
        .unwrap()
    {
        LookupResult::Hit(_) => {}
        other => panic!("expected Hit, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// as_reader — Arc 共有
// ---------------------------------------------------------------------------

#[test]
fn as_reader_shares_store() {
    let db = tmp_db();
    let writer = image_writer_with_db(&db);
    let file = TempFile::new(b"reader test");
    upsert_image(&writer, file.path());
    match writer.as_reader().lookup(file.path()).unwrap() {
        LookupResult::Hit(_) => {}
        other => panic!("expected Hit, got {:?}", other),
    }
}

#[test]
fn reader_clones_share_store() {
    let db = tmp_db();
    let writer = image_writer_with_db(&db);
    let file = TempFile::new(b"clone reader test");
    upsert_image(&writer, file.path());
    let r1 = writer.as_reader();
    let r2 = r1.clone();
    assert!(matches!(
        r1.lookup(file.path()).unwrap(),
        LookupResult::Hit(_)
    ));
    assert!(matches!(
        r2.lookup(file.path()).unwrap(),
        LookupResult::Hit(_)
    ));
}

// ---------------------------------------------------------------------------
// CacheRead trait 経由
// ---------------------------------------------------------------------------

#[test]
fn cache_read_trait_works_via_image_reader() {
    let db = tmp_db();
    let writer = image_writer_with_db(&db);
    let file = TempFile::new(b"trait test");
    upsert_image(&writer, file.path());
    let reader: &dyn CacheRead = &writer.as_reader();
    assert!(reader.check(file.path()).unwrap());
    assert_eq!(reader.list_paths().unwrap().len(), 1);
}

// ---------------------------------------------------------------------------
// スレッド並列 lookup
// ---------------------------------------------------------------------------

#[test]
fn parallel_lookup_with_threads() {
    let db = tmp_db();
    let writer = ImageCacheWriter::as_session(ImageCacheConfig {
        cache_config: CacheConfig {
            db_location: DbLocation::Custom(db.path().to_path_buf()),
            read_conns: 8,
            thumbnail_dir: None,
        },
    })
    .unwrap();

    let files: Vec<_> = (0..8)
        .map(|i| TempFile::new(format!("parallel image {i}").as_bytes()))
        .collect();
    for f in &files {
        upsert_image(&writer, f.path());
    }

    let reader = writer.as_reader();
    let hits = std::sync::Arc::new(std::sync::Mutex::new(0usize));

    std::thread::scope(|s| {
        let handles: Vec<_> = files
            .iter()
            .map(|f| {
                let r = reader.clone();
                let path = f.path.clone();
                let hits = hits.clone();
                s.spawn(move || {
                    if matches!(r.lookup(&path).unwrap(), LookupResult::Hit(_)) {
                        *hits.lock().unwrap() += 1;
                    }
                })
            })
            .collect();
        for h in handles {
            h.join().unwrap();
        }
    });

    assert_eq!(*hits.lock().unwrap(), files.len());
}

// ===========================================================================
// upsert_all / lookup_all (画像)
// ===========================================================================

#[test]
fn image_upsert_all_registers_all_files() {
    let db = tmp_db();
    let writer = image_writer_with_db(&db);
    let files: Vec<_> = (0..5)
        .map(|i| TempFile::new(format!("batch img {i}").as_bytes()))
        .collect();

    let reqs: Vec<_> = files
        .iter()
        .map(|f| UpsertImageRequest {
            path: f.path().to_path_buf(),
            clip_vector: Some(vec![1.0, 2.0]),
        })
        .collect();

    let results = writer.upsert_all(reqs);
    assert_eq!(results.len(), 5);
    for (_, r) in &results {
        assert!(r.is_ok(), "{r:?}");
    }
}

#[test]
fn image_upsert_all_partial_failure_continues() {
    let db = tmp_db();
    let writer = image_writer_with_db(&db);
    let good = TempFile::new(b"good image");
    let bad = std::path::PathBuf::from("/no/such/image.jpg");

    let results = writer.upsert_all(vec![
        UpsertImageRequest {
            path: good.path().to_path_buf(),
            clip_vector: None,
        },
        UpsertImageRequest {
            path: bad,
            clip_vector: None,
        },
    ]);

    assert_eq!(results.len(), 2);
    assert!(results[0].1.is_ok());
    assert!(results[1].1.is_err());
}

#[test]
fn image_lookup_all_returns_hits_for_upserted_files() {
    let db = tmp_db();
    let writer = image_writer_with_db(&db);
    let files: Vec<_> = (0..4)
        .map(|i| TempFile::new(format!("lookup_all {i}").as_bytes()))
        .collect();
    for f in &files {
        upsert_image(&writer, f.path());
    }

    let paths: Vec<&Path> = files.iter().map(|f| f.path()).collect();
    let results = writer.as_reader().lookup_all(&paths);

    assert_eq!(results.len(), 4);
    for (_, r) in &results {
        assert!(matches!(r.as_ref().unwrap(), LookupResult::Hit(_)));
    }
}

#[test]
fn image_lookup_all_returns_miss_for_unregistered_files() {
    let db = tmp_db();
    let writer = image_writer_with_db(&db);
    let files: Vec<_> = (0..3)
        .map(|i| TempFile::new(format!("miss_all {i}").as_bytes()))
        .collect();

    let paths: Vec<&Path> = files.iter().map(|f| f.path()).collect();
    let results = writer.as_reader().lookup_all(&paths);

    assert!(
        results
            .iter()
            .all(|(_, r)| matches!(r.as_ref().unwrap(), LookupResult::Miss))
    );
}

// ===========================================================================
// upsert_all / lookup_all (動画)
// ===========================================================================

#[test]
fn video_upsert_all_registers_all_files() {
    let (writer, _db) = video_writer();
    let files: Vec<_> = (0..4)
        .map(|i| TempFile::new(format!("batch vid {i}").as_bytes()))
        .collect();

    let reqs: Vec<_> = files
        .iter()
        .map(|f| UpsertVideoRequest {
            path: f.path().to_path_buf(),
            clip_vector: Some(vec![0.1, 0.2]),
            // wav2vec2_vector: Some(vec![0.3, 0.4]),
        })
        .collect();

    let results = writer.upsert_all(reqs);
    assert_eq!(results.len(), 4);
    for (_, r) in &results {
        assert!(r.is_ok(), "{r:?}");
    }
}

#[test]
fn video_upsert_all_partial_failure_continues() {
    let (writer, _db) = video_writer();
    let good = TempFile::new(b"good video");
    let bad = std::path::PathBuf::from("/no/such/video.mp4");

    let results = writer.upsert_all(vec![
        UpsertVideoRequest {
            path: good.path().to_path_buf(),
            clip_vector: None,
            // wav2vec2_vector: None,
        },
        UpsertVideoRequest {
            path: bad,
            clip_vector: None,
            // wav2vec2_vector: None,
        },
    ]);

    assert_eq!(results.len(), 2);
    assert!(results[0].1.is_ok());
    assert!(results[1].1.is_err());
}

#[test]
fn video_lookup_all_returns_hits_for_upserted_files() {
    let (writer, _db) = video_writer();
    let files: Vec<_> = (0..4)
        .map(|i| TempFile::new(format!("vid_lookup_all {i}").as_bytes()))
        .collect();

    let reqs: Vec<_> = files
        .iter()
        .map(|f| UpsertVideoRequest {
            path: f.path().to_path_buf(),
            clip_vector: Some(vec![1.0]),
            // wav2vec2_vector: None,
        })
        .collect();
    writer.upsert_all(reqs);

    let paths: Vec<&Path> = files.iter().map(|f| f.path()).collect();
    let results = writer.as_reader().lookup_all(&paths);

    assert_eq!(results.len(), 4);
    for (_, r) in &results {
        assert!(matches!(r.as_ref().unwrap(), LookupResult::Hit(_)));
    }
}
