use super::*;

#[test]
fn describe_locations_carries_the_grep_marker_and_all_three_paths() {
    let root = std::env::temp_dir().join(format!(
        "arama-describe-locations-test-{}",
        std::process::id()
    ));
    let locations = ResolvedLocations {
        settings_manager: ConfigManager::new().with_root_dir(root.join("settings")),
        local_dir: root.join("data"),
        cache_dir: root.join("cache"),
    };

    let described = describe_locations(&locations);

    assert!(described.starts_with("NATIVE_SMOKE_DATA_LOCATIONS_RESOLVED "));
    assert!(described.contains(&root.join("settings").display().to_string()));
    assert!(described.contains(&root.join("data").display().to_string()));
    assert!(described.contains(&root.join("cache").display().to_string()));
}

#[test]
fn describe_unresolved_carries_the_grep_marker_and_the_message() {
    let described = describe_unresolved("could not resolve the data location: no home dir");

    assert!(described.starts_with("NATIVE_SMOKE_DATA_LOCATIONS_UNRESOLVED: "));
    assert!(described.contains("could not resolve the data location"));
}

#[test]
fn move_or_copy_dir_moves_a_real_directory_tree() {
    let root = std::env::temp_dir().join(format!("arama-migrate-test-{}-move", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let from = root.join("from");
    let to = root.join("to");
    fs::create_dir_all(from.join("nested")).unwrap();
    fs::write(from.join("a.txt"), b"a").unwrap();
    fs::write(from.join("nested/b.txt"), b"b").unwrap();

    move_or_copy_dir(&from, &to).unwrap();

    assert!(!from.exists());
    assert_eq!(fs::read_to_string(to.join("a.txt")).unwrap(), "a");
    assert_eq!(fs::read_to_string(to.join("nested/b.txt")).unwrap(), "b");
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn migrate_directory_does_nothing_when_new_location_already_has_entries() {
    let root = std::env::temp_dir().join(format!(
        "arama-migrate-test-{}-both-populated",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    let legacy = root.join("legacy");
    let new = root.join("new");
    fs::create_dir_all(&legacy).unwrap();
    fs::write(legacy.join("old.txt"), b"old").unwrap();
    fs::create_dir_all(&new).unwrap();
    fs::write(new.join("current.txt"), b"current").unwrap();

    let mut notices = Vec::new();
    migrate_directory(MigrationKind::Data, Ok(legacy.clone()), &new, &mut notices);

    // New wins: untouched, and the legacy directory is left alone too.
    assert!(new.join("current.txt").exists());
    assert!(!new.join("old.txt").exists());
    assert!(legacy.join("old.txt").exists());
    assert!(notices.is_empty());
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn migrate_directory_moves_when_new_location_is_empty() {
    let root = std::env::temp_dir().join(format!(
        "arama-migrate-test-{}-empty-new",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    let legacy = root.join("legacy");
    let new = root.join("new");
    fs::create_dir_all(&legacy).unwrap();
    fs::write(legacy.join("model.bin"), b"weights").unwrap();
    fs::create_dir_all(&new).unwrap(); // exists, but empty

    let mut notices = Vec::new();
    migrate_directory(MigrationKind::Data, Ok(legacy.clone()), &new, &mut notices);

    assert!(new.join("model.bin").exists());
    assert!(!legacy.exists());
    assert!(notices.is_empty());
    fs::remove_dir_all(&root).unwrap();
}

/// Forces `move_or_copy_dir` to fail (the new location's parent is a
/// *file*, so nothing can be created under it) and checks the
/// resulting notice, for both `MigrationKind`s: no leftover
/// `{placeholder}` token, and both paths actually present in the
/// body, not just that it compiles.
///
/// English-only, deliberately: `app`'s test binary runs 60+ tests in
/// parallel, most of which assert exact English text without
/// expecting `arama_i18n`'s global locale to move under them. An
/// earlier version of this test looped over both locales; the same
/// change elsewhere in this crate (`core.rs`, `update/cache.rs`) was
/// found to race those tests when the full workspace suite ran
/// repeatedly, so this one was written English-only from the start.
/// `notice.migration_failed.body` (the key this reaches through
/// `migrate_directory`) is verified in both locales in
/// `arama-i18n`'s own, much smaller test binary instead
/// (`crates/i18n/src/lib.rs`'s `task_034_*` tests).
#[test]
fn migrate_directory_failure_notice_reads_correctly_in_english() {
    for kind in [MigrationKind::Data, MigrationKind::Cache] {
        let root = std::env::temp_dir().join(format!(
            "arama-migrate-test-{}-failure-{kind:?}",
            std::process::id(),
        ));
        let _ = fs::remove_dir_all(&root);
        let legacy = root.join("legacy");
        fs::create_dir_all(&legacy).unwrap();
        fs::write(legacy.join("a.txt"), b"a").unwrap();

        let blocker = root.join("blocker");
        fs::write(&blocker, b"not a directory").unwrap();
        let new = blocker.join("new");

        let mut notices = Vec::new();
        migrate_directory(kind, Ok(legacy.clone()), &new, &mut notices);

        assert_eq!(
            notices.len(),
            1,
            "{kind:?}: expected exactly one failure notice"
        );
        let notice = &notices[0];
        assert!(!notice.title.is_empty(), "{kind:?}");
        assert!(
            !notice.body.contains('{'),
            "{kind:?}: leftover placeholder in {}",
            notice.body
        );
        assert!(
            notice.body.contains(&legacy.display().to_string()),
            "{kind:?}: legacy path missing from {}",
            notice.body
        );
        assert!(
            notice.body.contains(&new.display().to_string()),
            "{kind:?}: new path missing from {}",
            notice.body
        );

        fs::remove_dir_all(&root).unwrap();
    }
}
