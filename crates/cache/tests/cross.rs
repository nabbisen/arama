// Cross-namespace, session-persistence, parallel-reader, and
// directory-summary integration tests for arama-cache.

#[path = "helpers.rs"]
#[allow(dead_code)]
mod helpers;

use arama_cache::{
    CacheConfig, CacheRead, DbLocation, ImageCacheConfig, ImageCacheWriter, LookupResult,
    UpsertImageRequest,
};

use helpers::{MINIMAL_JPEG, TempFile, files_in_dir, image_writer_with_db, tmp_db, upsert_image};

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
// onetime — file-backed DB persistence across instances
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
// as_reader — Arc-shared store
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
// CacheRead trait
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
// Parallel lookup
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

// ---------------------------------------------------------------------------
// RFC 004 — directory summaries and per-directory clearing
// ---------------------------------------------------------------------------

#[test]
fn summarize_by_dir_groups_and_aggregates() {
    let db = tmp_db();
    let writer = image_writer_with_db(&db);

    let dir_a = tempfile::TempDir::new().unwrap();
    let dir_b = tempfile::TempDir::new().unwrap();
    let a_files = files_in_dir(dir_a.path(), &[b"aaaa", b"bbbbbb", b"cc"]); // 4+6+2 = 12 bytes
    let b_files = files_in_dir(dir_b.path(), &[b"zzzzzzzz"]); // 8 bytes

    for p in a_files.iter().chain(b_files.iter()) {
        upsert_image(&writer, p);
    }

    let mut summaries = writer.as_reader().summarize_by_dir().unwrap();
    summaries.sort_by(|x, y| x.dir_path.cmp(&y.dir_path));
    assert_eq!(summaries.len(), 2);

    let canon_a = dir_a.path().canonicalize().unwrap();
    let a = summaries
        .iter()
        .find(|s| s.dir_path == canon_a.to_string_lossy())
        .expect("dir A summary present");
    assert_eq!(a.file_count, 3);
    assert_eq!(a.total_size, 12);
    assert!(0 < a.latest_cached_at, "timestamp recorded");

    let canon_b = dir_b.path().canonicalize().unwrap();
    let b = summaries
        .iter()
        .find(|s| s.dir_path == canon_b.to_string_lossy())
        .expect("dir B summary present");
    assert_eq!(b.file_count, 1);
    assert_eq!(b.total_size, 8);
}

#[test]
fn summarize_by_dir_empty_cache() {
    let db = tmp_db();
    let writer = image_writer_with_db(&db);
    let summaries = writer.as_reader().summarize_by_dir().unwrap();
    assert!(summaries.is_empty());
}

#[test]
fn delete_in_dir_removes_entries_and_thumbnails() {
    let thumb_dir = tempfile::TempDir::new().unwrap();
    let db = tmp_db();
    let writer = ImageCacheWriter::as_session(ImageCacheConfig {
        cache_config: CacheConfig {
            db_location: DbLocation::Custom(db.path().to_path_buf()),
            read_conns: 2,
            thumbnail_dir: Some(thumb_dir.path().to_path_buf()),
        },
    })
    .unwrap();

    let target_dir = tempfile::TempDir::new().unwrap();
    let sibling_dir = tempfile::TempDir::new().unwrap();
    // Real JPEG content so thumbnail generation succeeds.
    let target = target_dir.path().join("a.jpg");
    std::fs::write(&target, MINIMAL_JPEG).unwrap();
    let sibling = sibling_dir.path().join("b.jpg");
    std::fs::write(&sibling, MINIMAL_JPEG).unwrap();

    upsert_image(&writer, &target);
    upsert_image(&writer, &sibling);

    // Both thumbnails exist before clearing.
    let thumb_count_before = std::fs::read_dir(thumb_dir.path()).unwrap().count();
    assert_eq!(thumb_count_before, 2);

    let removed = writer.delete_in_dir(target_dir.path()).unwrap();
    assert_eq!(removed, 1);

    // Entry gone, sibling untouched.
    assert!(matches!(
        writer.lookup(&target).unwrap(),
        LookupResult::Miss
    ));
    assert!(matches!(
        writer.lookup(&sibling).unwrap(),
        LookupResult::Hit(_)
    ));

    // Target thumbnail removed; sibling's remains.
    let thumb_count_after = std::fs::read_dir(thumb_dir.path()).unwrap().count();
    assert_eq!(thumb_count_after, 1);
}

#[test]
fn delete_in_dir_is_not_recursive() {
    let db = tmp_db();
    let writer = image_writer_with_db(&db);

    let parent = tempfile::TempDir::new().unwrap();
    let child = parent.path().join("sub");
    std::fs::create_dir(&child).unwrap();

    let in_parent = parent.path().join("p.bin");
    std::fs::write(&in_parent, b"parent").unwrap();
    let in_child = child.join("c.bin");
    std::fs::write(&in_child, b"child").unwrap();

    upsert_image(&writer, &in_parent);
    upsert_image(&writer, &in_child);

    let removed = writer.delete_in_dir(parent.path()).unwrap();
    assert_eq!(removed, 1, "only the direct child entry is removed");

    assert!(matches!(
        writer.lookup(&in_parent).unwrap(),
        LookupResult::Miss
    ));
    assert!(matches!(
        writer.lookup(&in_child).unwrap(),
        LookupResult::Hit(_)
    ));
}
