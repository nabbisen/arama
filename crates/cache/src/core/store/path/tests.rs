// ---------------------------------------------------------------------------
// DB パス解決テスト
// ---------------------------------------------------------------------------

use crate::CacheConfig;

use super::*;

#[test]
fn resolve_prefers_config_db_path() {
    let db = tempfile::NamedTempFile::new().unwrap();
    let db_path = db.path().to_path_buf();
    let config = CacheConfig {
        db_path: Some(db_path.clone()),
        ..Default::default()
    };
    // 環境変数が設定されていても config.db_path が優先される
    // SAFETY: シングルスレッドのテスト内での設定
    unsafe {
        std::env::set_var("arama_cache_DB", "/should/not/be/used");
    }
    let resolved = resolve_db_path(&config);
    unsafe {
        std::env::remove_var("arama_cache_DB");
    }
    assert_eq!(resolved, db_path);
}

#[test]
fn resolve_prefers_env_var_over_xdg() {
    let db = tempfile::NamedTempFile::new().unwrap();
    let db_path = db.path().to_str().unwrap().to_string();
    let (_, db_path_buf) = db.keep().unwrap();

    let config = CacheConfig::default(); // 【変更】config を渡す
    // SAFETY: シングルスレッドのテスト内での設定
    unsafe {
        std::env::set_var("arama_cache_DB", &db_path);
    }
    let resolved = resolve_db_path(&config); // 【変更】引数追加
    unsafe {
        std::env::remove_var("arama_cache_DB");
    }
    let _ = std::fs::remove_file(&db_path_buf);

    assert_eq!(resolved.to_str().unwrap(), db_path);
}

#[test]
fn resolve_falls_back_without_env_var() {
    let config = CacheConfig::default(); // 【変更】config を渡す
    unsafe {
        std::env::remove_var("arama_cache_DB");
    }
    let resolved = resolve_db_path(&config); // 【変更】引数追加
    let s = resolved.to_str().unwrap();
    assert!(
        s.contains("cache.db") || s.contains("arama_cache.db"),
        "unexpected fallback path: {s}"
    );
}
