// ---------------------------------------------------------------------------
// DB パス解決テスト
// ---------------------------------------------------------------------------

use crate::config::db_location::DbLocation;

use super::*;

#[test]
fn db_location_custom_resolves_to_given_path() {
    let loc = DbLocation::Custom("/tmp/myapp/cache.db".into());
    assert_eq!(loc.resolve(), PathBuf::from("/tmp/myapp/cache.db"));
}

#[test]
fn db_location_workdir_default_filename() {
    let loc = DbLocation::WorkDir(None);
    assert_eq!(loc.resolve(), PathBuf::from("./arama_cache.db"));
}

#[test]
fn db_location_workdir_custom_filename() {
    let loc = DbLocation::WorkDir(Some("inference.db".into()));
    assert_eq!(loc.resolve(), PathBuf::from("./inference.db"));
}

#[test]
fn db_location_appcache_contains_exe_name_and_default_filename() {
    let loc = DbLocation::AppCache(None);
    let s = loc.resolve().to_str().unwrap().to_string();
    // 実行バイナリ名が含まれ、末尾が cache.db であること
    assert!(s.ends_with("cache.db"), "unexpected: {s}");
}

#[test]
fn db_location_appcache_custom_filename() {
    let loc = DbLocation::AppCache(Some("data.db".into()));
    let s = loc.resolve().to_str().unwrap().to_string();
    assert!(s.ends_with("data.db"), "unexpected: {s}");
}
