// Image-namespace integration tests for arama-cache.

#[path = "helpers.rs"]
#[allow(dead_code)]
mod helpers;

use std::path::Path;

use arama_cache::{
    CacheConfig, DbLocation, ImageCacheConfig, ImageCacheWriter, LookupResult, UpsertImageRequest,
};

use helpers::{MINIMAL_JPEG, TempFile, image_writer_with_db, tmp_db, upsert_image};

#[derive(Clone, Copy)]
struct TestImageFormat {
    name: &'static str,
    suffix: &'static str,
    format: image::ImageFormat,
}

const ACCEPTED_IMAGE_FORMATS: &[TestImageFormat] = &[
    TestImageFormat {
        name: "PNG",
        suffix: ".png",
        format: image::ImageFormat::Png,
    },
    TestImageFormat {
        name: "JPEG",
        suffix: ".jpg",
        format: image::ImageFormat::Jpeg,
    },
    TestImageFormat {
        name: "WebP",
        suffix: ".webp",
        format: image::ImageFormat::WebP,
    },
    TestImageFormat {
        name: "GIF",
        suffix: ".gif",
        format: image::ImageFormat::Gif,
    },
    TestImageFormat {
        name: "BMP",
        suffix: ".bmp",
        format: image::ImageFormat::Bmp,
    },
];

// ---------------------------------------------------------------------------
// upsert / lookup
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
    // Different byte length makes detection unambiguous: a size change
    // is caught by metadata comparison alone, regardless of timing.
    file.overwrite(b"modified!");
    assert!(matches!(
        writer.lookup(file.path()).unwrap(),
        LookupResult::Invalidated
    ));
}

// ---------------------------------------------------------------------------
// Thumbnail auto-generation
// ---------------------------------------------------------------------------

#[test]
fn image_thumbnail_generated_to_thumbnail_dir() {
    let thumb_dir = tempfile::TempDir::new().unwrap();
    let db = tmp_db();
    // Use a .jpg suffix so that image::open can infer the format from the
    // extension (real gallery files always have one; extensionless temp
    // files fail format detection even when the bytes are valid JPEG).
    let file = TempFile::with_suffix(MINIMAL_JPEG, ".jpg");

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

#[test]
fn image_thumbnail_generation_supports_accepted_formats() {
    for format in ACCEPTED_IMAGE_FORMATS {
        let source_dir = tempfile::TempDir::new().unwrap();
        let thumb_dir = tempfile::TempDir::new().unwrap();
        let db = tmp_db();
        let file = write_test_image(source_dir.path(), format);
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
                path: file.clone(),
                clip_vector: None,
            })
            .unwrap_or_else(|err| panic!("{} thumbnail generation failed: {err}", format.name));

        match writer.lookup(&file).unwrap() {
            LookupResult::Hit(entry) => {
                let thumb = entry
                    .thumbnail_path
                    .unwrap_or_else(|| panic!("{} thumbnail missing", format.name));
                assert!(
                    Path::new(&thumb).exists(),
                    "{} thumbnail file should exist: {thumb}",
                    format.name
                );
                assert!(
                    thumb.ends_with(".jpg"),
                    "{} generated thumbnail should be JPEG: {thumb}",
                    format.name
                );
                image::open(&thumb).unwrap_or_else(|err| {
                    panic!("{} generated JPEG did not decode: {err}", format.name)
                });
            }
            other => panic!("expected {} Hit, got {:?}", format.name, other),
        }
    }
}

// ---------------------------------------------------------------------------
// upsert_all / lookup_all
// ---------------------------------------------------------------------------

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

fn write_test_image(dir: &Path, format: &TestImageFormat) -> std::path::PathBuf {
    let path = dir.join(format!("fixture{}", format.suffix));
    let image = image::RgbImage::from_fn(4, 4, |x, y| {
        image::Rgb([
            (x * 47 + y * 11) as u8,
            (x * 19 + y * 29) as u8,
            (x * 7 + y * 53) as u8,
        ])
    });
    image
        .save_with_format(&path, format.format)
        .unwrap_or_else(|err| panic!("{} fixture encode failed: {err}", format.name));
    path
}
