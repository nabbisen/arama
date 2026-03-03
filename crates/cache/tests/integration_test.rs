//! インテグレーションテスト。

use std::io::Write;
use std::path::PathBuf;

use arama_cache::{
    ImageCacheConfig, ImageCacheWriter, LookupResult, UpsertImageRequest, UpsertVideoRequest,
    VideoCacheConfig, VideoCacheWriter,
};
use file_feature_cache::{CacheConfig, CacheWrite, DbLocation};

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
    fn path_str(&self) -> &str {
        self.path.to_str().unwrap()
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

fn image_writer() -> ImageCacheWriter {
    ImageCacheWriter::as_session(ImageCacheConfig {
        cache: CacheConfig {
            db_location: DbLocation::WorkDir(None),
            read_conns: 2,
            thumbnail_dir: None,
        },
        thumbnail: false,
    })
    .unwrap()
}

fn image_req(file_path: &str) -> UpsertImageRequest {
    UpsertImageRequest {
        file_path: file_path.to_string(),
        clip_vector: None,
    }
}

// ---------------------------------------------------------------------------
// 画像 upsert / lookup
// ---------------------------------------------------------------------------

#[test]
fn image_upsert_and_lookup_hit() {
    let writer = image_writer();
    let f = TempFile::new(b"image data");

    writer
        .upsert(UpsertImageRequest {
            file_path: f.path_str().to_string(),
            clip_vector: Some(vec![1.0, 2.0, 3.0]),
        })
        .unwrap();

    match writer.lookup(f.path_str()).unwrap() {
        LookupResult::Hit(entry) => {
            assert_eq!(entry.features.unwrap().clip_vector, vec![1.0f32, 2.0, 3.0]);
            assert!(entry.thumbnail_path.is_none());
        }
        other => panic!("expected Hit, got {:?}", other),
    }
}

#[test]
fn image_lookup_miss() {
    let writer = image_writer();
    assert!(matches!(
        writer.lookup("/no/such/file").unwrap(),
        LookupResult::Miss
    ));
}

#[test]
fn image_lookup_invalidated_on_change() {
    let writer = image_writer();
    let f = TempFile::new(b"original");
    writer.upsert(image_req(f.path_str())).unwrap();
    std::fs::write(&f.path, b"modified").unwrap();
    assert!(matches!(
        writer.lookup(f.path_str()).unwrap(),
        LookupResult::Invalidated
    ));
}

// ---------------------------------------------------------------------------
// 画像サムネイル自動生成
// ---------------------------------------------------------------------------

#[test]
fn image_thumbnail_generated_to_thumbnail_dir() {
    // 1×1 px の最小有効 JPEG
    let jpeg_bytes: &[u8] = &[
        0xff, 0xd8, 0xff, 0xe0, 0x00, 0x10, 0x4a, 0x46, 0x49, 0x46, 0x00, 0x01, 0x01, 0x00, 0x00,
        0x01, 0x00, 0x01, 0x00, 0x00, 0xff, 0xdb, 0x00, 0x43, 0x00, 0x08, 0x06, 0x06, 0x07, 0x06,
        0x05, 0x08, 0x07, 0x07, 0x07, 0x09, 0x09, 0x08, 0x0a, 0x0c, 0x14, 0x0d, 0x0c, 0x0b, 0x0b,
        0x0c, 0x19, 0x12, 0x13, 0x0f, 0x14, 0x1d, 0x1a, 0x1f, 0x1e, 0x1d, 0x1a, 0x1c, 0x1c, 0x20,
        0x24, 0x2e, 0x27, 0x20, 0x22, 0x2c, 0x23, 0x1c, 0x1c, 0x28, 0x37, 0x29, 0x2c, 0x30, 0x31,
        0x34, 0x34, 0x34, 0x1f, 0x27, 0x39, 0x3d, 0x38, 0x32, 0x3c, 0x2e, 0x33, 0x34, 0x32, 0xff,
        0xc0, 0x00, 0x0b, 0x08, 0x00, 0x01, 0x00, 0x01, 0x01, 0x01, 0x11, 0x00, 0xff, 0xc4, 0x00,
        0x1f, 0x00, 0x00, 0x01, 0x05, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b,
        0xff, 0xc4, 0x00, 0xb5, 0x10, 0x00, 0x02, 0x01, 0x03, 0x03, 0x02, 0x04, 0x03, 0x05, 0x05,
        0x04, 0x04, 0x00, 0x00, 0x01, 0x7d, 0x01, 0x02, 0x03, 0x00, 0x04, 0x11, 0x05, 0x12, 0x21,
        0x31, 0x41, 0x06, 0x13, 0x51, 0x61, 0x07, 0x22, 0x71, 0x14, 0x32, 0x81, 0x91, 0xa1, 0x08,
        0x23, 0x42, 0xb1, 0xc1, 0x15, 0x52, 0xd1, 0xf0, 0x24, 0x33, 0x62, 0x72, 0x82, 0x09, 0x0a,
        0x16, 0x17, 0x18, 0x19, 0x1a, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2a, 0x34, 0x35, 0x36, 0x37,
        0x38, 0x39, 0x3a, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x4a, 0x53, 0x54, 0x55, 0x56,
        0x57, 0x58, 0x59, 0x5a, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6a, 0x73, 0x74, 0x75,
        0x76, 0x77, 0x78, 0x79, 0x7a, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88, 0x89, 0x8a, 0x92, 0x93,
        0x94, 0x95, 0x96, 0x97, 0x98, 0x99, 0x9a, 0xa2, 0xa3, 0xa4, 0xa5, 0xa6, 0xa7, 0xa8, 0xa9,
        0xaa, 0xb2, 0xb3, 0xb4, 0xb5, 0xb6, 0xb7, 0xb8, 0xb9, 0xba, 0xc2, 0xc3, 0xc4, 0xc5, 0xc6,
        0xc7, 0xc8, 0xc9, 0xca, 0xd2, 0xd3, 0xd4, 0xd5, 0xd6, 0xd7, 0xd8, 0xd9, 0xda, 0xe1, 0xe2,
        0xe3, 0xe4, 0xe5, 0xe6, 0xe7, 0xe8, 0xe9, 0xea, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7,
        0xf8, 0xf9, 0xfa, 0xff, 0xda, 0x00, 0x08, 0x01, 0x01, 0x00, 0x00, 0x3f, 0x00, 0xfb, 0x6d,
        0xff, 0xd9,
    ];

    let thumb_dir = tempfile::TempDir::new().unwrap();
    let db = tempfile::NamedTempFile::new().unwrap();
    let f = TempFile::new(jpeg_bytes);

    let writer = ImageCacheWriter::as_session(ImageCacheConfig {
        cache: CacheConfig {
            db_location: DbLocation::Custom(db.path().to_path_buf()),
            read_conns: 1,
            thumbnail_dir: Some(thumb_dir.path().to_path_buf()),
        },
        thumbnail: true,
    })
    .unwrap();

    writer
        .upsert(UpsertImageRequest {
            file_path: f.path_str().to_string(),
            clip_vector: None,
        })
        .unwrap();

    match writer.lookup(f.path_str()).unwrap() {
        LookupResult::Hit(entry) => {
            let thumb = entry.thumbnail_path.expect("thumbnail_path should be Some");
            assert!(
                std::path::Path::new(&thumb).exists(),
                "thumbnail file should exist"
            );
            assert!(thumb.ends_with(".jpg"));
        }
        other => panic!("expected Hit, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// 動画 upsert / lookup
// ---------------------------------------------------------------------------

#[test]
fn video_upsert_and_lookup_hit() {
    let writer = VideoCacheWriter::as_session(VideoCacheConfig {
        cache: CacheConfig {
            db_location: DbLocation::WorkDir(None),
            read_conns: 2,
            thumbnail_dir: None,
        },
        thumbnail: false,
        ffmpeg_path: None,
    })
    .unwrap();

    let f = TempFile::new(b"video data");
    writer
        .upsert(UpsertVideoRequest {
            file_path: f.path_str().to_string(),
            clip_vector: Some(vec![1.0, 2.0]),
            wav2vec2_vector: Some(vec![3.0, 4.0]),
        })
        .unwrap();

    match writer.lookup(f.path_str()).unwrap() {
        LookupResult::Hit(entry) => {
            let feat = entry.features.unwrap();
            assert_eq!(feat.clip_vector, vec![1.0f32, 2.0]);
            assert_eq!(feat.wav2vec2_vector, vec![3.0f32, 4.0]);
        }
        other => panic!("expected Hit, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// delete / verify_or_invalidate / list_paths
// ---------------------------------------------------------------------------

#[test]
fn delete_removes_entry() {
    let writer = image_writer();
    let f = TempFile::new(b"to delete");
    writer.upsert(image_req(f.path_str())).unwrap();
    assert!(writer.delete(f.path_str()).unwrap());
    assert!(matches!(
        writer.lookup(f.path_str()).unwrap(),
        LookupResult::Miss
    ));
}

#[test]
fn verify_or_invalidate_true_if_unchanged() {
    let writer = image_writer();
    let f = TempFile::new(b"stable");
    writer.upsert(image_req(f.path_str())).unwrap();
    assert!(writer.verify_or_invalidate(f.path_str()).unwrap());
}

#[test]
fn list_paths_returns_all_registered() {
    let writer = image_writer();
    let files: Vec<_> = (0..3)
        .map(|i| TempFile::new(format!("f{i}").as_bytes()))
        .collect();
    for f in &files {
        writer.upsert(image_req(f.path_str())).unwrap();
    }
    assert_eq!(writer.list_paths().unwrap().len(), 3);
}

// ---------------------------------------------------------------------------
// oneshot / as_reader
// ---------------------------------------------------------------------------

#[test]
fn oneshot_persists_across_instances() {
    let db = tempfile::NamedTempFile::new().unwrap();
    let loc = DbLocation::Custom(db.path().to_path_buf());
    let f = TempFile::new(b"oneshot test");

    ImageCacheWriter::oneshot(loc.clone())
        .unwrap()
        .upsert(image_req(f.path_str()))
        .unwrap();

    match ImageCacheWriter::oneshot(loc)
        .unwrap()
        .lookup(f.path_str())
        .unwrap()
    {
        LookupResult::Hit(_) => {}
        other => panic!("expected Hit, got {:?}", other),
    }
}

#[test]
fn as_reader_shares_store() {
    let writer = image_writer();
    let f = TempFile::new(b"reader test");
    writer.upsert(image_req(f.path_str())).unwrap();

    let reader = writer.as_reader();
    assert!(matches!(
        reader.lookup(f.path_str()).unwrap(),
        LookupResult::Hit(_)
    ));
}
