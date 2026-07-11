// Cache-footprint and manual-prune integration tests (RFC 016).

#[path = "helpers.rs"]
#[allow(dead_code)]
mod helpers;

use std::path::Path;
use std::time::Duration;

use arama_cache::{
    CacheConfig, CacheMaintenance, CachePruneRequest, DbLocation, ImageCacheConfig,
    ImageCacheWriter, LookupResult,
};

use helpers::{MINIMAL_JPEG, TempFile, image_writer_with_db, tmp_db, upsert_image};

fn maintenance(db: &tempfile::NamedTempFile, thumb_dir: &Path) -> CacheMaintenance {
    CacheMaintenance::as_session(CacheConfig {
        db_location: DbLocation::Custom(db.path().to_path_buf()),
        read_conns: 2,
        thumbnail_dir: Some(thumb_dir.to_path_buf()),
    })
    .unwrap()
}

#[test]
fn footprint_counts_database_sidecars_and_thumbnail_dir() {
    let db = tmp_db();
    let thumb_dir = tempfile::TempDir::new().unwrap();
    std::fs::write(thumb_dir.path().join("a.jpg"), b"123").unwrap();
    std::fs::create_dir(thumb_dir.path().join("nested")).unwrap();
    std::fs::write(thumb_dir.path().join("nested").join("b.jpg"), b"12345").unwrap();
    std::fs::write(sidecar(db.path(), "-wal"), b"1234567").unwrap();
    std::fs::write(sidecar(db.path(), "-shm"), b"12345678901").unwrap();

    let footprint = maintenance(&db, thumb_dir.path()).footprint().unwrap();

    assert_eq!(footprint.database_bytes, 0);
    assert_eq!(footprint.database_sidecar_bytes, 18);
    assert_eq!(footprint.thumbnail_bytes, 8);
    assert_eq!(footprint.total_bytes, 26);
}

#[test]
fn prune_removes_orphans_then_oldest_recorded_thumbnail() {
    let db = tmp_db();
    let thumb_dir = tempfile::TempDir::new().unwrap();
    let writer = ImageCacheWriter::as_session(ImageCacheConfig {
        cache_config: CacheConfig {
            db_location: DbLocation::Custom(db.path().to_path_buf()),
            read_conns: 2,
            thumbnail_dir: Some(thumb_dir.path().to_path_buf()),
        },
    })
    .unwrap();

    let old = TempFile::with_suffix(MINIMAL_JPEG, ".jpg");
    let new = TempFile::with_suffix(MINIMAL_JPEG, ".jpg");
    upsert_image(&writer, old.path());
    std::thread::sleep(Duration::from_secs(1));
    upsert_image(&writer, new.path());

    let old_thumb = match writer.lookup(old.path()).unwrap() {
        LookupResult::Hit(entry) => entry.thumbnail_path.expect("old thumbnail"),
        other => panic!("expected old hit, got {other:?}"),
    };
    let new_thumb = match writer.lookup(new.path()).unwrap() {
        LookupResult::Hit(entry) => entry.thumbnail_path.expect("new thumbnail"),
        other => panic!("expected new hit, got {other:?}"),
    };
    let old_thumb_bytes = std::fs::metadata(&old_thumb).unwrap().len();
    let orphan = thumb_dir.path().join("orphan.jpg");
    std::fs::write(&orphan, b"orphan-bytes").unwrap();
    let orphan_bytes = std::fs::metadata(&orphan).unwrap().len();

    let maintenance = maintenance(&db, thumb_dir.path());
    let before = maintenance.footprint().unwrap();
    let max_bytes = before
        .total_bytes
        .saturating_sub(orphan_bytes)
        .saturating_sub(old_thumb_bytes);
    let report = maintenance.prune(CachePruneRequest { max_bytes }).unwrap();

    assert_eq!(report.removed_orphan_thumbnail_bytes, orphan_bytes);
    assert_eq!(report.removed_entries, 1);
    assert_eq!(report.removed_recorded_thumbnail_bytes, old_thumb_bytes);
    assert!(report.after.thumbnail_bytes < report.before.thumbnail_bytes);
    assert!(!orphan.exists());
    assert!(!Path::new(&old_thumb).exists());
    assert!(Path::new(&new_thumb).exists());
    assert!(matches!(
        writer.lookup(old.path()).unwrap(),
        LookupResult::Miss
    ));
    assert!(matches!(
        writer.lookup(new.path()).unwrap(),
        LookupResult::Hit(_)
    ));
}

#[test]
fn prune_reports_unreachable_target_without_deleting_database_only_entries() {
    let db = tmp_db();
    let writer = image_writer_with_db(&db);
    let file = TempFile::new(b"database only");
    upsert_image(&writer, file.path());

    let maintenance = CacheMaintenance::as_session(CacheConfig {
        db_location: DbLocation::Custom(db.path().to_path_buf()),
        read_conns: 2,
        thumbnail_dir: None,
    })
    .unwrap();
    let report = maintenance
        .prune(CachePruneRequest { max_bytes: 0 })
        .unwrap();

    assert!(!report.target_reached);
    assert!(0 < report.unreclaimable_bytes);
    assert_eq!(report.removed_entries, 0);
    assert!(matches!(
        writer.lookup(file.path()).unwrap(),
        LookupResult::Hit(_)
    ));
}

#[cfg(unix)]
#[test]
fn prune_treats_recorded_thumbnail_delete_failure_as_best_effort() {
    use std::os::unix::fs::PermissionsExt;

    let db = tmp_db();
    let thumb_dir = tempfile::TempDir::new().unwrap();
    let writer = ImageCacheWriter::as_session(ImageCacheConfig {
        cache_config: CacheConfig {
            db_location: DbLocation::Custom(db.path().to_path_buf()),
            read_conns: 2,
            thumbnail_dir: Some(thumb_dir.path().to_path_buf()),
        },
    })
    .unwrap();

    let file = TempFile::with_suffix(MINIMAL_JPEG, ".jpg");
    upsert_image(&writer, file.path());

    let thumb = match writer.lookup(file.path()).unwrap() {
        LookupResult::Hit(entry) => entry.thumbnail_path.expect("thumbnail"),
        other => panic!("expected hit, got {other:?}"),
    };
    let thumb_path = Path::new(&thumb);

    std::fs::set_permissions(thumb_dir.path(), std::fs::Permissions::from_mode(0o500)).unwrap();
    let result = maintenance(&db, thumb_dir.path()).prune(CachePruneRequest { max_bytes: 0 });
    std::fs::set_permissions(thumb_dir.path(), std::fs::Permissions::from_mode(0o700)).unwrap();

    let report = result.unwrap();
    assert_eq!(report.removed_entries, 1);
    assert_eq!(report.removed_recorded_thumbnail_bytes, 0);
    assert!(!report.target_reached);
    assert!(thumb_path.exists());
    assert!(matches!(
        writer.lookup(file.path()).unwrap(),
        LookupResult::Miss
    ));
}

fn sidecar(path: &Path, suffix: &str) -> std::path::PathBuf {
    let mut os = path.as_os_str().to_os_string();
    os.push(suffix);
    os.into()
}
